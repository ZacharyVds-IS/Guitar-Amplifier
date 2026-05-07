use crate::infrastructure::file_loader::file_loader_trait::FileLoaderTrait;
use hound::{SampleFormat, WavReader};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use tracing::{info, warn};

pub struct FileLoader;

impl FileLoader {
    pub fn new() -> Self {
        Self
    }
}

impl FileLoaderTrait for FileLoader {
    fn read_wav_sample_rate(&self, path: &Path) -> Option<u32> {
        WavReader::open(path).ok().map(|reader| reader.spec().sample_rate)
    }

    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32> {
        match WavReader::open(path) {
            Ok(mut reader) => {
                let spec = reader.spec();
                match spec.sample_format {
                    SampleFormat::Float => {
                        match reader.samples::<f32>().collect::<Result<Vec<_>, _>>() {
                            Ok(buffer) => {
                                info!(
                                    "Loaded IR '{}' (channels={}, sample_rate={}, samples={})",
                                    path.display(),
                                    spec.channels,
                                    spec.sample_rate,
                                    buffer.len()
                                );
                                buffer
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to read float samples from '{}': {e}",
                                    path.display()
                                );
                                Vec::new()
                            }
                        }
                    }
                    SampleFormat::Int => {
                        let max = ((1_i64 << (spec.bits_per_sample.saturating_sub(1))) - 1) as f32;
                        match reader
                            .samples::<i32>()
                            .map(|sample| sample.map(|value| value as f32 / max.max(1.0)))
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(buffer) => {
                                info!(
                                    "Loaded IR '{}' (channels={}, sample_rate={}, samples={})",
                                    path.display(),
                                    spec.channels,
                                    spec.sample_rate,
                                    buffer.len()
                                );
                                buffer
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to read int samples from '{}': {e}",
                                    path.display()
                                );
                                Vec::new()
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to open IR file '{}': {e}", path.display());
                Vec::new()
            }
        }
    }

    fn list_ir_profile_file_names(&self, directory: &Path) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(directory)
            .map_err(|e| format!("Failed to read directory '{}': {e}", directory.display()))?;

        let mut names: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_file() {
                    return None;
                }

                 if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("wav"))
                    != Some(true)
                {
                    return None;
                }

                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
            })
            .collect();

        names.sort();
        Ok(names)
    }

    fn ensure_directory(&self, directory: &Path) -> Result<(), String> {
        fs::create_dir_all(directory)
            .map_err(|e| format!("Failed to create directory '{}': {e}", directory.display()))
    }

    fn write_file_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        fs::write(path, bytes)
            .map_err(|e| format!("Failed to write file '{}': {e}", path.display()))
    }

    fn remove_file(&self, path: &Path) -> Result<(), String> {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file '{}': {e}", path.display()))
    }

    fn validate_ir_wav_bytes(
        &self,
        file_name: &str,
        file_bytes: &[u8],
        impulse_threshold: f32,
    ) -> Result<(), String> {
        if !file_name.to_ascii_lowercase().ends_with(".wav") {
            return Err("Only .wav IR files are supported".to_string());
        }

        let mut reader = WavReader::new(Cursor::new(file_bytes)).map_err(|e| {
            let raw = e.to_string();
            if raw.contains("unexpected fmt chunk size") {
                format!(
                    "Unsupported WAV format for '{}': {}. Re-export as PCM 16/24-bit or IEEE float 32-bit WAV.",
                    file_name, raw
                )
            } else {
                format!("Invalid WAV file '{}': {raw}", file_name)
            }
        })?;

        let spec = reader.spec();

        const IMPULSE_SEARCH_WINDOW_SAMPLES: usize = 256;

        let max_abs_in_window = match spec.sample_format {
            SampleFormat::Float => {
                let mut iter = reader.samples::<f32>();
                let first = iter
                    .next()
                    .ok_or_else(|| "IR file is empty".to_string())
                    .and_then(|s| s.map_err(|e| format!("Failed to read first sample: {e}")))?;

                let mut max_abs = first.abs();
                for sample in iter.take(IMPULSE_SEARCH_WINDOW_SAMPLES.saturating_sub(1)) {
                    let value = sample.map_err(|e| format!("Failed to read WAV samples: {e}"))?;
                    max_abs = max_abs.max(value.abs());
                }

                max_abs
            }
            SampleFormat::Int => {
                let max = ((1_i64 << (spec.bits_per_sample.saturating_sub(1))) - 1) as f32;
                let mut iter = reader.samples::<i32>();
                let first = iter
                    .next()
                    .ok_or_else(|| "IR file is empty".to_string())
                    .and_then(|s| s.map_err(|e| format!("Failed to read first sample: {e}")))?;

                let mut max_abs = (first as f32 / max.max(1.0)).abs();
                for sample in iter.take(IMPULSE_SEARCH_WINDOW_SAMPLES.saturating_sub(1)) {
                    let value = sample.map_err(|e| format!("Failed to read WAV samples: {e}"))?;
                    max_abs = max_abs.max((value as f32 / max.max(1.0)).abs());
                }

                max_abs
            }
        };

        if max_abs_in_window <= impulse_threshold {
            return Err(
                "Invalid IR: no impulse detected at file start (first 256 samples are effectively silent)"
                    .to_string(),
            );
        }

        Ok(())
    }
}

