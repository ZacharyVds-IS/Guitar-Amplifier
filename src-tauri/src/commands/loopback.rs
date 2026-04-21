use crate::services::audio_service::AudioService;

#[tauri::command]
pub(crate) fn start_loopback(audio_service: tauri::State<AudioService>) {
    audio_service.start_loopback();
}