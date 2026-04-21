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
        self.gain.store(gain, Ordering::Relaxed);
    }

    pub fn gain_handle(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }
}
