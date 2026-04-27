use crate::domain::tone_stack_dto::ToneStackDto;
use crate::services::audio_service::AudioService;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

/// Represents the complete amplifier configuration state.
///
/// This DTO is serialized to JSON and sent to the frontend to display the current
/// settings of the amplifier, including gain, master volume, and active/inactive status.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    /// The current input gain level.
    pub gain: f32,
    /// The current master volume level.
    pub master_volume: f32,
    /// Whether the audio loopback is currently active.
    pub is_active: bool,
    /// The current tone stack settings, including bass, mid, treble.
    pub tone_stack: ToneStackDto
}

impl AmpConfigDto {
    /// Constructs an `AmpConfigDto` from the current state of an [`AudioService`].
    ///
    /// Reads atomic values from the service's channel and master volume with relaxed memory ordering.
    ///
    /// # Arguments
    ///
    /// * `service` - The [`AudioService`] to snapshot.
    pub fn from_service(service: &AudioService) -> Self {
        let channel = service.channel();

        Self {
            gain: channel.gain().load(Ordering::Relaxed),
            master_volume: service.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active(),
            tone_stack: ToneStackDto{
                bass: channel.tone_stack().bass().load(Ordering::Relaxed),
                middle: channel.tone_stack().middle().load(Ordering::Relaxed),
                treble: channel.tone_stack().treble().load(Ordering::Relaxed)
            }
        }
    }
}