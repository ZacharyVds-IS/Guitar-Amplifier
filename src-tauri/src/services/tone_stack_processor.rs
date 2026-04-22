use std::sync::Arc;
use atomic_float::AtomicF32;

pub struct ToneStackProcessor {
    bass: Arc<AtomicF32>,
    mid: Arc<AtomicF32>,
    treble: Arc<AtomicF32>,
}

impl ToneStackProcessor {
    pub fn new(bass: Arc<AtomicF32>, mid: Arc<AtomicF32>, treble: Arc<AtomicF32>) -> Self {
        Self { bass, mid, treble }
    }
}