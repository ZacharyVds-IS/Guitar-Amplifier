use crate::domain::amp_config_dto::AmpConfigDto;
use crate::domain::tone_stack_dto::ToneStackDto;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;

/// Retrieves the current amplifier configuration as an [`AmpConfigDto`].
///
/// This command captures the state of gain, master volume, and other parameters
/// from the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// Returns `Ok(AmpConfigDto)` on success, or `Err(String)` if the service state cannot be locked.
#[tauri::command]
pub fn get_amp_config(
    audio_service: tauri::State<'_, Mutex<AudioService>>
) -> Result<AmpConfigDto, String> {
    let service = audio_service.lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    Ok(AmpConfigDto::from_service(&service))
}

/// Toggles the audio loopback on or off.
///
/// Delegates to [`AudioService::toggle_loopback`] to start or stop audio processing.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `is_on` - Whether to enable (`true`) or disable (`false`) the loopback.
#[tauri::command]
pub(crate) fn toggle_on_off(audio_service: tauri::State<Mutex<AudioService>>, is_on: bool) {
    let mut service = audio_service.inner().lock().unwrap();
    service.toggle_loopback(is_on);
}

/// Sets the input gain level for the amplifier.
///
/// Applies the gain value to the [`Channel`] within the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `gain` - The gain value (must be a positive value).
///
/// [`Channel`]: crate::domain::channel::Channel
#[tauri::command]
pub(crate) fn set_gain(audio_service: tauri::State<Mutex<AudioService>>, gain: f32) {
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_gain(gain);
}

/// Sets the master volume level for the amplifier.
///
/// Applies the master volume value to the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `master_volume` - The master volume value (must be positive).
///
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_master_volume(audio_service: tauri::State<Mutex<AudioService>>, master_volume: f32) {
    let service = audio_service.inner().lock().unwrap();
    service.set_master_volume(master_volume);
}

#[tauri::command]
pub(crate) fn set_tone_stack(audio_service: tauri::State<Mutex<AudioService>>, tone_stack: ToneStackDto){
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_tone_stack(tone_stack);
}

#[tauri::command]
pub(crate) fn set_bass(audio_service: tauri::State<Mutex<AudioService>>, bass: f32){
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_bass(bass);
}

#[tauri::command]
pub(crate) fn set_middle(audio_service: tauri::State<Mutex<AudioService>>, middle: f32){
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_middle(middle);
}

#[tauri::command]
pub(crate) fn set_treble(audio_service: tauri::State<Mutex<AudioService>>, treble: f32){
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_treble(treble);
}

#[tauri::command]
pub(crate) fn set_volume(audio_service: tauri::State<Mutex<AudioService>>, volume: f32){
    let service = audio_service.inner().lock().unwrap();
    service.channels().get(*service.current_channel_index()).unwrap().set_volume(volume);
}