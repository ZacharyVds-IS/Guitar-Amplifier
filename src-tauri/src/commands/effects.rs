use crate::services::audio_service::AudioService;
use crate::services::effects::effect_type::EffectType;
use std::sync::{Arc, Mutex};

#[tauri::command]
pub async fn add_effect(
    effect_type: EffectType,
    state: tauri::State<'_, Arc<Mutex<AudioService>>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|_| "Failed to lock service")?;

    let target_channel_id = *service.current_channel_id();


    if let Some(channel) = service.channels_mut().iter_mut().find(|c| c.id() == target_channel_id) {
        let effect = effect_type.create(channel.next_effect_id());
        channel.add_effect_to_chain(effect);
        Ok(())
    } else {
        Err("Channel not found".into())
    }
}

#[tauri::command]
pub(crate) fn remove_effect(audio_service: tauri::State<Mutex<AudioService>>, effect_id: u32) {
    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = *service.current_channel_id();
    let current_channel = service.channels_mut().iter_mut().find(|c| c.id() == channel_id).unwrap();
    current_channel.remove_effect_from_chain(effect_id);
}