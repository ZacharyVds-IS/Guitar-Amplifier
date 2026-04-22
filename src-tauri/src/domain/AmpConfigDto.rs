use serde::{Serialize, Deserialize};
use std::sync::atomic::Ordering;
use crate::services::audio_service::AudioService;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    pub gain: f32,
    pub master_volume: f32,
    pub is_active: bool,
}

impl AmpConfigDto {
    pub fn from_service(service: &AudioService) -> Self {
        let channel = service.channel();

        Self {
            gain: channel.gain().load(Ordering::Relaxed),
            master_volume: channel.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active()
        }
    }
}