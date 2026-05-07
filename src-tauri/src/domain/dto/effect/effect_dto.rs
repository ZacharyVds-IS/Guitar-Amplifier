use crate::domain::dto::effect::delay_dto::DelayDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::effects::delay::delay::Delay;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use serde::{Deserialize, Serialize};

/// A serialisable, tagged representation of any effect in the signal chain.
///
/// Uses serde's adjacently-tagged format so that both the Rust serialisation
/// and the TypeScript typegen agree on the wire shape:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EffectDto {
    /// Hard-clipping distortion effect.
    HCDistortion(HcDistortionDto),
    Delay(DelayDto)
}

impl EffectDto {
    pub fn add_to_domain(self, next_effect_id: u32, sample_rate: u32) -> Box<dyn Effect> {
        match self { 
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.color,
                sample_rate,
                20,
                0.95
            ))
        }
    }

    pub fn to_domain(self, sample_rate: u32) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.color,
                sample_rate,
                dto.delay_time,
                dto.level,
            ))
        }
    }
}