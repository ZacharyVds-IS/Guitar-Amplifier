use std::sync::Mutex;
use crate::domain::AmpConfigDto::AmpConfigDto;
use crate::services::audio_service::AudioService;

#[tauri::command]
pub fn get_amp_config(
    audio_service: tauri::State<'_, Mutex<AudioService>>
) -> Result<AmpConfigDto, String> {
    let service = audio_service.lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    Ok(AmpConfigDto::from_service(&service))
}

#[tauri::command]
pub(crate) fn toggle_on_off(audio_service: tauri::State<Mutex<AudioService>>, is_on: bool) {
    let mut service = audio_service.inner().lock().unwrap();
    service.toggle_loopback(is_on);
}


#[tauri::command]
pub(crate) fn set_gain(audio_service: tauri::State<Mutex<AudioService>>, gain: f32) {
    let service = audio_service.inner().lock().unwrap();
    service.channel().set_gain(gain);
}

#[tauri::command]
pub(crate) fn set_master_volume(audio_service: tauri::State<Mutex<AudioService>>, master_volume: f32) {
    let service = audio_service.inner().lock().unwrap();
    service.channel().set_master_volume(master_volume);
}