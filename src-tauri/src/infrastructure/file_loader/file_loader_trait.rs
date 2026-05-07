use std::path::Path;

pub trait FileLoaderTrait: Send + Sync {
    fn read_wav_sample_rate(&self, path: &Path) -> Option<u32>;
    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32>;
    fn list_ir_profile_file_names(&self, directory: &Path) -> Result<Vec<String>, String>;
}

