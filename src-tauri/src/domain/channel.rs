use atomic_float::AtomicF32;
use derive_getters::Getters;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Getters, Clone)]
pub struct Channel {
    name: String,
    gain: Arc<AtomicF32>,
}

impl Channel {
    pub fn new(name: String, gain: f32) -> Self {
        Self {
            name,
            gain: Arc::new(AtomicF32::new(gain)),
        }
    }

    pub fn set_gain(&self, gain: f32) {
        if gain.is_sign_positive() {
            self.gain.store(gain, Ordering::Relaxed);
        } else {
            //TODO: Log error
            panic!("Gain must be positive");
        }
    }

    pub fn gain_handle(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
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
            let channel = Channel::new("Test".to_string(), 1.0);
            channel.set_gain(0.5);
            assert_eq!(channel.gain_handle().load(Ordering::Relaxed), 0.5);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Gain must be positive")]
        fn gain_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), 1.0);
            channel.set_gain(-0.5);
            assert_eq!(channel.gain_handle().load(Ordering::Relaxed), 0.5);
        }
    }
}