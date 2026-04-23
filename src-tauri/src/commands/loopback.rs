use std::sync::Mutex;
use crate::services::audio_service::AudioService;

/// Starts the audio loopback on a dedicated background thread.
///
/// Delegates to [`AudioService::start_loopback`] to begin capturing and processing audio.
/// If the loopback is already running, this command is a no-op.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state, accessed via Tauri's state management.
#[tauri::command]
pub(crate) fn start_loopback(audio_service: tauri::State<'_, Mutex<AudioService>>) {
    let mut service = audio_service.lock().unwrap();
    service.start_loopback();
}
