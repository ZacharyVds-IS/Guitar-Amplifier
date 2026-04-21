use crate::services::audio_service::AudioService;

#[tauri::command]
pub(crate) fn set_gain(audio_service: tauri::State<AudioService>, gain: f32) {
    audio_service.channel().set_gain(gain);
}