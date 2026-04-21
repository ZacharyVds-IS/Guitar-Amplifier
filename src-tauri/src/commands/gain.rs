use std::sync::Mutex;
use tracing::info;
use crate::services::audio_service::AudioService;

#[tauri::command]
pub fn set_gain(audio_service: tauri::State<Mutex<AudioService>>, gain: f32) {
    info!("Setting gain to {}", gain);

    let mut service = audio_service.lock().unwrap();
    service.channel().set_gain(gain);
}