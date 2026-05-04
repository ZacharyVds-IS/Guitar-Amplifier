use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// DTO for [`HCDistortion`](crate::services::effects::distortion::hc_distortion::HCDistortion).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct HcDistortionDto {
    pub id: u32,
    pub name: String,
    pub is_active: bool,
    /// UI colour for the pedal chassis (hex string, e.g. `"#e67e22"`).
    pub color: String,
    /// Hard-clip threshold in the range `(0.0, 1.0]`.
    /// Lower values produce heavier distortion.
    pub threshold: f32,
}
