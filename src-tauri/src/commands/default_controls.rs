use crate::services::audio_service::AudioService;

#[tauri::command]
pub(crate) fn set_gain(audio_service: tauri::State<AudioService>, gain: f32) {
    audio_service.channel().set_gain(gain);
}

#[tauri::command]
pub(crate) fn set_master_volume(audio_service: tauri::State<AudioService>, master_volume: f32) {
    audio_service.channel().set_master_volume(master_volume);
}