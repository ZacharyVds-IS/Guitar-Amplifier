use std::sync::Arc;
use std::sync::atomic::Ordering;
use atomic_float::AtomicF32;
use tracing::info;
use crate::domain::audio_processor::AudioProcessor;

pub struct GainProcessor {
    gain: Arc<AtomicF32>,
    current: f32
}

impl GainProcessor {
    pub fn new(gain: Arc<AtomicF32>) -> Self {
        info!("initi gain processor");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use atomic_float::AtomicF32;
    use std::sync::atomic::Ordering;

    fn make_processor(initial_gain: f32) -> GainProcessor {
        let gain = Arc::new(AtomicF32::new(initial_gain));
        GainProcessor::new(gain)
    }


    #[cfg(test)]
    mod success_path {
        use super::*;
        #[test]
        fn transition_to_target_should_be_smooth_up() {
            let gain = Arc::new(AtomicF32::new(0.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(1.0, Ordering::Relaxed);

            for _ in 0..5_000 {
                processor.process(1.0);
            }

            assert!(processor.current > 0.9);
            assert!(processor.current < 1.0);
        }

        #[test]
        fn transition_to_target_should_be_smooth_down() {
            let gain = Arc::new(AtomicF32::new(1.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(0.0, Ordering::Relaxed);

            for _ in 0..5_000 {
                processor.process(1.0);
            }

            assert!(processor.current < 0.1);
            assert!(processor.current > 0.0);
        }

        #[test]
        fn steady_state_does_not_change_value() {
            let mut processor = make_processor(1.0);

            for _ in 0..1_000 {
                processor.process(1.0);
            }

            assert!((processor.current - 1.0).abs() < 1e-6);
        }

        #[test]
        fn output_is_scaled_by_current_gain() {
            let gain = Arc::new(AtomicF32::new(1.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(0.5, Ordering::Relaxed);

            for _ in 0..1_000 {
                processor.process(1.0);
            }

            let output = processor.process(1.0);

            assert!((output - processor.current).abs() < 1e-6);
        }
    }
    //This part of the code does not have a failure path
}