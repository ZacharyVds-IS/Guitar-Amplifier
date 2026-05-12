use serde::{Deserialize, Serialize};

/// Analyzer metadata shared with the frontend to avoid IPC contract drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumContractDto {
    pub live_spectrum_event: String,
    pub min_db: f32,
    pub max_db: f32,
    pub min_frequency_hz: f32,
    pub max_frequency_hz: f32,
}
