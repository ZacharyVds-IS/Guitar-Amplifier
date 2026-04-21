use std::sync::Mutex;
use crate::services::audio_service::AudioService;

#[tauri::command]
pub(crate) fn start_loopback(audio_service: tauri::State<'_, Mutex<AudioService>>) {
    let mut service = audio_service.lock().unwrap();
    service.start_loopback();
}
