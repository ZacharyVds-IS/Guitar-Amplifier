use serde::{Deserialize, Serialize};
use crate::domain::tone_stack_dto::ToneStackDto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelDto {
    /// Name of the Channel
    pub name: String,
    /// The input gain level of the Channel.
    pub gain: f32,
    /// The tone stack settings, including bass, mid, treble of the Channel.
    pub tone_stack: ToneStackDto,
    /// The volume of the Channel.
    pub volume: f32,
}