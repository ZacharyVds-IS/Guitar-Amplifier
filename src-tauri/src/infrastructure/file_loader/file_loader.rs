use crate::infrastructure::file_loader::file_loader_trait::FileLoaderTrait;
use hound::{SampleFormat, WavReader};
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
}

