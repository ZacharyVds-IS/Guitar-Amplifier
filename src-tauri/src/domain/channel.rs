use atomic_float::AtomicF32;
use derive_getters::Getters;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::error;
use crate::domain::tone_stack::ToneStack;

#[derive(Clone)]
pub struct Channel {
    name: String,
    gain: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
    tone_stack: Arc<ToneStack>
}

impl Channel {
    pub fn new(name: String, gain: Option<f32>, master_volume: Option<f32>) -> Self {
        let gain = gain.unwrap_or(1.0);
        let master_volume = master_volume.unwrap_or(1.0);

        Self {
            name,
            gain: Arc::new(AtomicF32::new(gain)),
            master_volume: Arc::new(AtomicF32::new(master_volume)),
            tone_stack: Arc::new(ToneStack::new()),
        }
    }

    pub fn set_gain(&self, gain: f32) {
        if gain.is_sign_positive() {
            self.gain.store(gain, Ordering::Relaxed);
        } else {
            error!("Gain must be a positive number");
            panic!("Gain must be positive");
        }
    }

    pub fn set_master_volume(&self, master_volume: f32) {
        if master_volume.is_sign_positive() {
            self.master_volume.store(master_volume, Ordering::Relaxed);
        } else {
            error!("Master volume must be a positive number");
            panic!("Master volume must be positive");
        }
    }
    pub fn set_tone_stack(&mut self, tone_stack: Arc<ToneStack>) {
        self.tone_stack = tone_stack;
    }

    pub fn gain(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }

    pub fn master_volume(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.master_volume)
    }

    pub fn tone_stack(&self) -> &Arc<ToneStack> {
        &self.tone_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;
        #[test]
        fn gain_set_to_positive_value_should_succeed() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_gain(0.5);
            assert_eq!(channel.gain().load(Ordering::Relaxed), 0.5);
        }
        #[test]
        fn master_volume_set_to_positive_value_should_succeed() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_master_volume(0.5);
            assert_eq!(channel.master_volume().load(Ordering::Relaxed), 0.5);
        }

    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Gain must be positive")]
        fn gain_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_gain(-0.5);
        }
        #[test]
        #[should_panic(expected = "Master volume must be positive")]
        fn master_volume_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_master_volume(-0.5);
        }
    }
}