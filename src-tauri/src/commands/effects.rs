use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

/// Toggles an effect's active state on the current channel.
///
/// The change takes effect on the very next audio sample — no loopback restart needed.
///
/// # Arguments
/// * `effect_id` — ID of the effect to toggle.
///
/// # Returns
/// The new active state (`true` = processing, `false` = bypassed).
#[tauri::command]
pub fn toggle_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    effect_id: u32,
) -> Result<bool, String> {
    let service = audio_service.lock().map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    let new_state = channel.toggle_effect(effect_id)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        is_active = new_state,
        "Effect toggled"
    );
    Ok(new_state)
}

/// Sets the threshold parameter on an `HCDistortion` effect.
///
/// The value is clamped to `[0.001, 1.0]` by the effect itself.
/// The change takes effect on the very next audio sample.
///
/// # Arguments
/// * `effect_id`  — ID of the HCDistortion effect.
/// * `threshold`  — New clip threshold in the range `(0.0, 1.0]`.
#[tauri::command]
pub fn set_hc_distortion_threshold(
    audio_service: tauri::State<Mutex<AudioService>>,
    effect_id: u32,
    threshold: f32,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "threshold", threshold)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        threshold,
        "HCDistortion threshold updated"
    );
    Ok(())
}



