use atomic_float::AtomicF32;
use derive_getters::Getters;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::error;

#[derive(Clone)]
pub struct Channel {
    name: String,
    gain: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
    bass: Arc<AtomicF32>,
    middle: Arc<AtomicF32>,
    treble: Arc<AtomicF32>,
}

impl Channel {
    pub fn new(name: String, gain: Option<f32>, master_volume: Option<f32>) -> Self {
        let gain = gain.unwrap_or(1.0);
        let master_volume = master_volume.unwrap_or(1.0);

        Self {
            name,
            gain: Arc::new(AtomicF32::new(gain)),
            master_volume: Arc::new(AtomicF32::new(master_volume)),
            bass: Arc::new(AtomicF32::new(1.0)),
            middle: Arc::new(AtomicF32::new(1.0)),
            treble: Arc::new(AtomicF32::new(1.0)),
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

    pub fn set_bass(&self, bass: f32) {
        if bass.is_sign_positive() && bass <= 1.0 {
            self.bass.store(bass, Ordering::Relaxed);
        } else {
            error!("Bass must be a positive number between 0 and 1");
            panic!("Bass must be positive and between 0 and 1");
        }
    }

    pub fn set_middle(&self, middle: f32) {
        if middle.is_sign_positive() && middle <= 1.0 {
            self.middle.store(middle, Ordering::Relaxed);
        }else {
            error!("Middle must be a positive number between 0 and 1");
            panic!("Middle must be positive and between 0 and 1");
        }
    }

    pub fn set_treble(&self, treble: f32) {
    if treble.is_sign_positive() && treble <= 1.0{
        self.treble.store(treble, Ordering::Relaxed);
    } else {
            error!("Treble must be a positive number between 0 and 1");
            panic!("Treble must be positive and between 0 and 1");
        }
    }

    pub fn gain(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }

    pub fn master_volume(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.master_volume)
    }

    pub fn bass(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.bass)
    }

    pub fn middle(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.middle)
    }

    pub fn treble(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.treble)
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
        #[test]
        fn bass_set_to_positive_value_within_range_should_succeed() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_bass(0.5);
            assert_eq!(channel.bass().load(Ordering::Relaxed), 0.5);
        }
        #[test]
        fn middle_set_to_positive_value_within_range_should_succeed() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_middle(0.5);
            assert_eq!(channel.middle().load(Ordering::Relaxed), 0.5);
        }
        #[test]
        fn treble_set_to_positive_value_within_range_should_succeed() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_treble(0.5);
            assert_eq!(channel.treble().load(Ordering::Relaxed), 0.5);
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
        #[test]
        #[should_panic(expected = "Bass must be positive and between 0 and 1")]
        fn bass_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_bass(-0.5);
        }
        #[test]
        #[should_panic(expected = "Bass must be positive and between 0 and 1")]
        fn bass_set_to_value_greater_than_one_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_bass(1.5);
        }
        #[test]
        #[should_panic(expected = "Middle must be positive and between 0 and 1")]
        fn middle_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_middle(-0.5);
        }
        #[test]
        #[should_panic(expected = "Middle must be positive and between 0 and 1")]
        fn middle_set_to_value_greater_than_one_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_middle(1.5);
        }
        #[test]
        #[should_panic(expected = "Treble must be positive and between 0 and 1")]
        fn treble_set_to_negative_value_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_treble(-0.5);
        }
        #[test]
        #[should_panic(expected = "Treble must be positive and between 0 and 1")]
        fn treble_set_to_value_greater_than_one_should_panic() {
            let channel = Channel::new("Test".to_string(), None, None);
            channel.set_treble(1.5);
        }
    }
}