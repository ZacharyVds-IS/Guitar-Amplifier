use std::path::Path;

pub trait FileLoaderTrait: Send + Sync {
    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32>;
}

