use std::path::Path;

pub trait FileLoaderTrait: Send + Sync {
    fn read_wav_sample_rate(&self, path: &Path) -> Option<u32>;
    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32>;
    fn list_ir_profile_file_names(&self, directory: &Path) -> Result<Vec<String>, String>;
    fn ensure_directory(&self, directory: &Path) -> Result<(), String>;
    fn write_file_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), String>;
    fn remove_file(&self, path: &Path) -> Result<(), String>;
    fn validate_ir_wav_bytes(
        &self,
        file_name: &str,
        file_bytes: &[u8],
        impulse_threshold: f32,
    ) -> Result<(), String>;
}

