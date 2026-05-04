use crate::domain::effect::Effect;
use serde::{Deserialize, Serialize};
/// Data transfer object for an Effect's settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectDto {
    /// Unique identifier for the Effect.
    pub id: u32,
    /// Name of the Effect
    pub name: String,
    /// True if the effect is currently active and processing audio, false if bypassed.
    pub is_active: bool,
    /// Color of the pedal in the UI, used for display purposes. This is a string representation of the color (e.g., hex code).
    pub color: String,
}


impl From<&dyn Effect> for EffectDto {
    fn from(effect: &dyn Effect) -> Self {
        Self {
            id: effect.id(),
            name: effect.name().to_string(),
            is_active: effect.is_active(),
            color: effect.get_color(),
        }
    }
}

