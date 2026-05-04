use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

/// Toggles an effect's active state on the current channel.
/// Enables or disables audio processing for a specific effect. The change takes effect
/// on the very next audio sample — no loopback restart needed.
///
/// This is a generic command that works with any effect type.
///
/// # Arguments
/// * `effect_id` — Unique ID of the effect to toggle
///
/// # Returns
/// * `Ok(bool)` — The new active state (`true` = processing, `false` = bypassed)
/// * `Err(String)` — Error message if effect ID is invalid or channel not found
///
/// # Implementation Details
///
/// - Updates the effect's [`Arc<AtomicBool>`] active flag
/// - Changes apply immediately to audio processing thread
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

/// # Sets the Clipping Threshold on an HC Distortion Effect
///
/// Adjusts the Drive parameter: lower thresholds produce heavier distortion.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `threshold` — Clipping level in range `(0.0, 1.0]`
///                 * Values < 0.001 are clamped to 0.001
///                 * Values > 1.0 are clamped to 1.0
///
/// # Returns
/// * `Ok(())` — Threshold updated successfully
/// * `Err(String)` — Error if effect not found or parameter update fails

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

/// # Sets the Output Level (Boost) on an HC Distortion Effect
///
/// Adjusts the Level parameter: controls output amplitude after clipping.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `level` — Normalised level in range `[0.0, 1.0]`
///            * `0.0` = unity gain (no boost)
///            * `1.0` = ×2.0 boost
///            * Values outside range are clamped
///
/// # Returns
/// * `Ok(())` — Level updated successfully
/// * `Err(String)` — Error if effect not found or parameter update fails

#[tauri::command]
pub fn set_hc_distortion_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    effect_id: u32,
    level: f32,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    let gain = 1.0 + level.clamp(0.0, 1.0);
    channel.set_effect_param(effect_id, "level", gain)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        level,
        "HCDistortion level updated"
    );
    Ok(())
}


