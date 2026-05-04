use crate::domain::dto::effect::flip_effect_dto::FlipEffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use serde::{Deserialize, Serialize};

/// A serialisable, tagged representation of any effect in the signal chain.
///
/// Uses serde's adjacently-tagged format so that both the Rust serialisation
/// and the TypeScript typegen agree on the wire shape:
///
/// ```json
/// { "kind": "Flip",         "data": { "id": 1, "name": "Flip",  "is_active": true,  "color": "#e74c3c" } }
/// { "kind": "HCDistortion", "data": { "id": 2, "name": "Dist",  "is_active": false, "color": "#e67e22", "threshold": 0.5 } }
/// ```
///
/// Adding a new effect = new variant + new `*Dto` struct in its own file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EffectDto {
    /// Phase-flip (polarity inversion) effect.
    Flip(FlipEffectDto),
    /// Hard-clipping distortion effect.
    HCDistortion(HcDistortionDto),
}
