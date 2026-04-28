use serde::{Deserialize, Serialize};

/// Represents the measured round-trip latency from input through DSP to output and back to input.
///
/// This is a real-world measurement that captures the actual end-to-end delay including:
/// - Input/output buffer delays
/// - Hardware AD/DA conversion delay
/// - OS scheduling delays
/// - Driver delays
/// - Resampler buffering
/// - DSP processing time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct RoundTripLatencyDto {
    /// Measured round-trip latency in milliseconds
    pub latency_ms: f64,
    /// Whether the measurement was successful
    pub is_valid: bool,
    /// Optional error message if measurement failed
    pub error: Option<String>,
}

impl RoundTripLatencyDto {
    pub fn success(latency_ms: f64) -> Self {
        Self {
            latency_ms,
            is_valid: true,
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            latency_ms: 0.0,
            is_valid: false,
            error: Some(error),
        }
    }
}

