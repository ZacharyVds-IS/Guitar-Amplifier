use serde::{Deserialize, Serialize};

/// Represents the I/O buffer latency contribution of the current audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct BufferLatencyDto {
    /// Input-side buffering delay in milliseconds.
    pub input_buffer_latency_ms: f64,
    /// Output-side buffering delay in milliseconds.
    pub output_buffer_latency_ms: f64,
    /// Sum of input + output buffering delay in milliseconds.
    pub total_buffer_latency_ms: f64,
}

impl BufferLatencyDto {
    pub fn new(input_buffer_latency_ms: f64, output_buffer_latency_ms: f64) -> Self {
        Self {
            input_buffer_latency_ms,
            output_buffer_latency_ms,
            total_buffer_latency_ms: input_buffer_latency_ms + output_buffer_latency_ms,
        }
    }
}

