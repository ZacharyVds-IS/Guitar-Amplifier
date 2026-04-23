use atomic_float::AtomicF32;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::error;
use crate::domain::tone_stack::ToneStack;
use crate::domain::tone_stack_dto::ToneStackDto;

/// Represents an audio channel with atomic gain and master volume parameters.
///
/// `Channel` uses [`AtomicF32`] for lock-free updates of audio parameters from
/// the UI thread while the audio processing thread reads them without waiting.
/// This enables low-latency parameter changes without interrupting audio playback.
///
/// Both gain and master volume are validated to be positive values; attempting to
/// set a negative value will panic.
#[derive(Clone)]
pub struct Channel {
    name: String,
    gain: Arc<AtomicF32>,
    master_volume: Arc<AtomicF32>,
    tone_stack: Arc<ToneStack>
}

impl Channel {
    /// Creates a new `Channel` with the given name and optional gain/master volume values.
    ///
    /// If `gain` or `master_volume` are not provided, they default to `1.0`.
    ///
    /// # Arguments
    ///
    /// * `name` - A human-readable name for the channel (e.g., "Main", "Overdrive").
    /// * `gain` - Optional initial gain value. Defaults to `1.0` if `None`.
    /// * `master_volume` - Optional initial master volume value. Defaults to `1.0` if `None`.
    pub fn new(name: String, gain: Option<f32>, master_volume: Option<f32> ) -> Self {
        let gain = gain.unwrap_or(1.0);
        let master_volume = master_volume.unwrap_or(1.0);

        Self {
            name,
            gain: Arc::new(AtomicF32::new(gain)),
            master_volume: Arc::new(AtomicF32::new(master_volume)),
            tone_stack: Arc::new(ToneStack::new()),
        }
    }

    /// Sets the gain value for this channel.
    ///
    /// The gain value is atomically updated and will be read by the audio processing
    /// thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `gain` - The new gain value. Must be positive (> 0.0).
    ///
    /// # Panics
    ///
    /// Panics if `gain` is negative or zero.
    pub fn set_gain(&self, gain: f32) {
        if gain.is_sign_positive() {
            self.gain.store(gain, Ordering::Relaxed);
        } else {
            error!("Gain must be a positive number");
            panic!("Gain must be positive");
        }
    }

    /// Sets the master volume value for this channel.
    ///
    /// The master volume value is atomically updated and will be read by the audio processing
    /// thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `master_volume` - The new master volume value. Must be positive (> 0.0).
    ///
    /// # Panics
    ///
    /// Panics if `master_volume` is negative or zero.
    pub fn set_master_volume(&self, master_volume: f32) {
        if master_volume.is_sign_positive() {
            self.master_volume.store(master_volume, Ordering::Relaxed);
        } else {
            error!("Master volume must be a positive number");
            panic!("Master volume must be positive");
        }
    }
    pub fn set_tone_stack(&self, tone_stack: ToneStackDto) {
        self.tone_stack.set_bass(tone_stack.bass);
        self.tone_stack.set_middle(tone_stack.middle);
        self.tone_stack.set_treble(tone_stack.treble);
    }

    pub fn set_bass(&self, bass: f32) {
        self.tone_stack.set_bass(bass/100.0);
    }

    pub fn set_middle(&self, middle: f32) {
        self.tone_stack.set_middle(middle/100.0);
    }

    pub fn set_treble(&self, treble: f32) {
        self.tone_stack.set_treble(treble/100.0);
    }

    /// Returns a cloned [`Arc`] to the atomic gain value.
    ///
    /// Allows independent threads to share and read/write the gain parameter
    /// without contention.
    pub fn gain(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }

    /// Returns a cloned [`Arc`] to the atomic master volume value.
    ///
    /// Allows independent threads to share and read/write the master volume parameter
    /// without contention.
    pub fn master_volume(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.master_volume)
    }

    pub fn tone_stack(&self) -> Arc<ToneStack> {
        Arc::clone(&self.tone_stack)
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
