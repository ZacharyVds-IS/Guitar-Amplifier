use crate::domain::tone_stack_dto::ToneStackDto;
use crate::services::audio_service::AudioService;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use crate::domain::channel_dto::ChannelDto;

/// Represents the complete amplifier configuration state.
///
/// This DTO is serialized to JSON and sent to the frontend to display the current
/// settings of the amplifier, including gain, master volume, and active/inactive status.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    /// The current master volume level.
    pub master_volume: f32,
    /// Whether the audio loopback is currently active.
    pub is_active: bool,
    /// The current active channel.
    pub current_channel: ChannelDto,
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
        let channel = service.channels().get(*service.current_channel_index()).unwrap();

        Self {
            master_volume: service.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active(),
            current_channel: ChannelDto{
                name: channel.name().clone(),
                gain: channel.gain().load(Ordering::Relaxed),
                tone_stack: ToneStackDto{
                    bass: channel.tone_stack().bass().load(Ordering::Relaxed),
                    middle: channel.tone_stack().middle().load(Ordering::Relaxed),
                    treble: channel.tone_stack().treble().load(Ordering::Relaxed)
                },
                volume: channel.volume().load(Ordering::Relaxed),
            },

        }
    }
}