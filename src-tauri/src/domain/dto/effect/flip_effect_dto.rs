use serde::{Deserialize, Serialize};
use ts_rs::TS;
/// DTO for [`FlipEffect`](crate::services::effects::flip_effect::FlipEffect).
//
// No effect-specific parameters beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize,TS)]
#[ts(export)]
pub struct FlipEffectDto {
    pub id: u32,
    pub name: String,
    pub is_active: bool,
    /// UI colour for the pedal chassis (hex string, e.g. `"#e74c3c"`).
    pub color: String,
}

