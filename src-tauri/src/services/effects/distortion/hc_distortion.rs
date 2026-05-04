use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Hard-clipping distortion effect.
///
/// Any sample whose absolute value exceeds `threshold` is clamped to `±threshold`,
/// producing the flat-top waveform characteristic of hard clipping.
///
/// Both `is_active` and `limit` are stored as `Arc<Atomic*>` so the audio thread
/// and command handlers share them without any lock.
pub struct HCDistortion {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    /// Clip level in `(0.0, 1.0]`.
    limit: Arc<AtomicF32>,
    /// UI chassis colour (hex string, e.g. `"#e67e22"`).
    color: String,
}

impl HCDistortion {
    pub fn new(id: u32, name: String, is_active: bool, threshold: f32, color: String) -> Self {
        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            limit: Arc::new(AtomicF32::new(threshold.clamp(0.001, 1.0))),
            color,
        }
    }

    pub fn threshold(&self) -> f32 { self.limit.load(Ordering::Relaxed) }

    pub fn set_threshold(&self, threshold: f32) {
        self.limit.store(threshold.clamp(0.001, 1.0), Ordering::Relaxed);
    }
}

impl AudioProcessor for HCDistortion {
    fn process(&mut self, sample: f32) -> f32 {
        let limit = self.limit.load(Ordering::Relaxed);
        sample.clamp(-limit, limit)
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
        map
    }

    fn to_dto(&self) -> EffectDto {
        EffectDto::HCDistortion(HcDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            threshold: self.limit.load(Ordering::Relaxed),
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
        HCDistortion::new(0, "HC".to_string(), true, threshold, "#e67e22".to_string())
    }

    mod success_path {
        use super::*;

        #[test]
        fn sample_within_threshold_is_unchanged() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(0.3), 0.3);
            assert_eq!(fx.process(-0.3), -0.3);
        }

        #[test]
        fn sample_above_threshold_is_clipped() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(0.9), 0.5);
        }

        #[test]
        fn process_if_active_clips_when_active() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process_if_active(0.9), 0.5);
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
            assert_eq!(fx.process(0.9), 0.3);
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