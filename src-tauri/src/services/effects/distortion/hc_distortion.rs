use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::processors::gain::gain_processor::GainProcessor;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Hard-clipping distortion effect.
///
/// Any sample whose absolute value exceeds `threshold` is clamped to `±threshold`,
/// producing the flat-top waveform characteristic of hard clipping.
/// After clipping the signal is passed through a [`GainProcessor`] whose gain is
/// controlled by `level` — a normalised value in `[0.0, 1.0]` that maps to a
/// linear boost of `1.0` (unity, no boost) .. `2.0` (double amplitude).
///
/// `is_active`, `limit`, and `level` are stored as `Arc<Atomic*>` so the audio
/// thread and command handlers share them without any lock.
pub struct HCDistortion {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    /// Clip level in `(0.0, 1.0]`.
    limit: Arc<AtomicF32>,
    /// Internal gain atomic shared with `level_gain`. Stores [1.0, 2.0].
    level: Arc<AtomicF32>,
    /// GainProcessor that applies the smoothed level boost after clipping.
    level_gain: GainProcessor,
    /// UI chassis colour (hex string, e.g. `"#e67e22"`).
    color: String,
}

impl HCDistortion {
    /// * `threshold` — clip level in `(0.0, 1.0]`
    /// * `level`     — normalised output boost in `[0.0, 1.0]`
    ///                 (`0.0` = unity gain, `1.0` = ×2.0 boost)
    pub fn new(id: u32, name: String, is_active: bool, threshold: f32, level: f32, color: String) -> Self {
        let gain_value = 1.0 + level.clamp(0.0, 1.0); // map [0,1] → [1,2]
        let level_arc = Arc::new(AtomicF32::new(gain_value));
        let level_gain = GainProcessor::new(Arc::clone(&level_arc));
        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            limit: Arc::new(AtomicF32::new(threshold.clamp(0.001, 1.0))),
            level: level_arc,
            level_gain,
            color,
        }
    }

    pub fn threshold(&self) -> f32 { self.limit.load(Ordering::Relaxed) }

    pub fn set_threshold(&self, threshold: f32) {
        self.limit.store(threshold.clamp(0.001, 1.0), Ordering::Relaxed);
    }

    /// Returns the normalised level `[0.0, 1.0]` (reverses the internal [1, 2] mapping).
    pub fn level(&self) -> f32 {
        (self.level.load(Ordering::Relaxed) - 1.0).clamp(0.0, 1.0)
    }

    /// Sets the level from a normalised value `[0.0, 1.0]`.
    pub fn set_level(&self, level: f32) {
        self.level.store(1.0 + level.clamp(0.0, 1.0), Ordering::Relaxed);
    }
}

impl AudioProcessor for HCDistortion {
    fn process(&mut self, sample: f32) -> f32 {
        let limit = self.limit.load(Ordering::Relaxed);
        let clipped = sample.clamp(-limit, limit);
        self.level_gain.process(clipped)
    }
}

impl Effect for HCDistortion {
    fn id(&self) -> u32 { self.id }
    fn name(&self) -> &str { &self.name }
    fn get_color(&self) -> String { self.color.clone() }
    fn active_flag(&self) -> Arc<AtomicBool> { Arc::clone(&self.is_active) }

    fn f32_params(&self) -> HashMap<&'static str, Arc<AtomicF32>> {
        let mut map = HashMap::new();
        map.insert("threshold", Arc::clone(&self.limit));
        // "level" stores the internal gain [1, 2]; the command converts [0,1] before writing.
        map.insert("level", Arc::clone(&self.level));
        map
    }

    fn to_dto(&self) -> EffectDto {
        EffectDto::HCDistortion(HcDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            threshold: self.limit.load(Ordering::Relaxed),
            level: self.level(),
        })
    }

    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active() { self.process(sample) } else { sample }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distortion(threshold: f32) -> HCDistortion {
        HCDistortion::new(0, "HC".to_string(), true, threshold, 0.0, "#e67e22".to_string())
    }

    mod success_path {
        use super::*;

        #[test]
        fn sample_within_threshold_is_unchanged() {
            let mut fx = distortion(0.5);
            // With level=0.0 the gain processor targets 1.0; after many samples it converges.
            // For a quick unit check, drive it to steady-state first.
            for _ in 0..10_000 { fx.process(0.0); }
            assert!((fx.process(0.3) - 0.3).abs() < 1e-3);
            assert!((fx.process(-0.3) - (-0.3)).abs() < 1e-3);
        }

        #[test]
        fn sample_above_threshold_is_clipped() {
            let mut fx = distortion(0.5);
            for _ in 0..10_000 { fx.process(0.0); }
            assert!((fx.process(0.9) - 0.5).abs() < 1e-3);
        }

        #[test]
        fn process_if_active_clips_when_active() {
            let mut fx = distortion(0.5);
            for _ in 0..10_000 { fx.process(0.0); }
            assert!((fx.process_if_active(0.9) - 0.5).abs() < 1e-3);
        }

        #[test]
        fn process_if_active_passes_through_when_inactive() {
            let mut fx = distortion(0.5);
            fx.set_active(false);
            assert_eq!(fx.process_if_active(0.9), 0.9);
        }

        #[test]
        fn set_threshold_updates_clip_level() {
            let mut fx = distortion(0.8);
            fx.set_threshold(0.3);
            assert!((fx.threshold() - 0.3).abs() < 1e-6);
            for _ in 0..10_000 { fx.process(0.0); }
            assert!((fx.process(0.9) - 0.3).abs() < 1e-3);
        }

        #[test]
        fn level_boost_doubles_output_at_max() {
            let mut fx = HCDistortion::new(0, "HC".to_string(), true, 1.0, 1.0, "#e67e22".to_string());
            // Converge gain processor to ×2.0
            for _ in 0..20_000 { fx.process(0.0); }
            let out = fx.process(0.3);
            assert!((out - 0.6).abs() < 0.01, "expected ≈0.6, got {out}");
        }

        #[test]
        fn level_unity_at_zero() {
            let mut fx = distortion(1.0); // level=0.0
            for _ in 0..10_000 { fx.process(0.0); }
            let out = fx.process(0.4);
            assert!((out - 0.4).abs() < 0.01, "expected ≈0.4, got {out}");
        }
    }

    mod failure_path {
        use super::*;

        #[test]
        fn threshold_above_one_is_clamped_to_one() {
            let fx = distortion(2.0);
            assert_eq!(fx.threshold(), 1.0);
        }

        #[test]
        fn threshold_of_zero_is_clamped_to_minimum() {
            let fx = distortion(0.0);
            assert!(fx.threshold() > 0.0);
        }
    }
}