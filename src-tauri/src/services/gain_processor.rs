use std::sync::Arc;
use std::sync::atomic::Ordering;
use atomic_float::AtomicF32;
use crate::domain::audio_processor::AudioProcessor;

pub struct GainProcessor {
    gain: Arc<AtomicF32>,
    current: f32
}

impl GainProcessor {
    pub fn new(gain: Arc<AtomicF32>) -> Self {
        let initial = gain.load(Ordering::Relaxed);
        Self {
            gain,
            current: initial,
        }
    }
}

impl AudioProcessor for GainProcessor {
    fn process(&mut self, sample: f32) -> f32 {
        let target = self.gain.load(Ordering::Relaxed);

        // Simple one-pole smoothing
        self.current += (target - self.current) * 0.001;

        sample * self.current
    }
}