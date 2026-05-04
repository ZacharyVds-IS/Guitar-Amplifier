use crate::domain::effect::Effect;
use crate::services::effects::flip_effect::FlipEffect;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EffectConfig {
    pub name: String,
    pub color: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EffectType {
    Flip(EffectConfig),
}

impl EffectType {
    pub fn create(&self, next_id: u32) -> Box<dyn Effect> {
        match self {
            EffectType::Flip(config) => {
                Box::new(FlipEffect::new(
                    next_id,
                    config.name.clone(),
                    config.color.clone()
                ))
            }
        }
    }
}