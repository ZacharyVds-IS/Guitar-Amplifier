use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumSnapshotDto {
    pub sample_rate_hz: u32,
    pub frequencies_hz: Vec<f32>,
    pub magnitudes: Vec<f32>,
    pub level_db: f32,
}
