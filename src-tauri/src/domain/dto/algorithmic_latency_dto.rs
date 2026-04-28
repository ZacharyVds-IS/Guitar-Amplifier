use serde::{Deserialize, Serialize};

/// Represents algorithmic latency for a single audio processor in the DSP chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct AlgorithmicLatencyDto {
    /// Name of the audio processor (e.g., "Gain", "Tone Stack", "Master Volume").
    pub processor_name: String,

    /// Algorithmic delay contributed by the processor in audio samples.
    pub latency_samples: u32,

    /// Algorithmic delay in milliseconds at the current output sample rate.
    pub latency_ms: f64,
}

impl AlgorithmicLatencyDto {
    pub fn new(processor_name: impl Into<String>, latency_samples: u32, sample_rate_hz: u32) -> Self {
        let latency_ms = if sample_rate_hz == 0 {
            0.0
        } else {
            (latency_samples as f64 / sample_rate_hz as f64) * 1000.0
        };

        Self {
            processor_name: processor_name.into(),
            latency_samples,
            latency_ms,
        }
    }
}

