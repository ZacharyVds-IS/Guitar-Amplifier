use crate::domain::tone_stack_dto::ToneStackDto;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;

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
    let service = audio_service.lock().unwrap();
    service.channel().set_master_volume(master_volume);
}

#[tauri::command]
pub(crate) fn set_tone_stack(audio_service: tauri::State<Mutex<AudioService>>, tone_stack: ToneStackDto){
    let service = audio_service.lock().unwrap();
    service.channel().set_tone_stack(tone_stack);
}

#[tauri::command]
pub(crate) fn set_bass(audio_service: tauri::State<Mutex<AudioService>>, bass: f32){
    let service = audio_service.lock().unwrap();
    service.channel().set_bass(bass);
}

#[tauri::command]
pub(crate) fn set_middle(audio_service: tauri::State<Mutex<AudioService>>, middle: f32){
    let service = audio_service.lock().unwrap();
    service.channel().set_middle(middle);
}

#[tauri::command]
pub(crate) fn set_treble(audio_service: tauri::State<Mutex<AudioService>>, treble: f32){
    let service = audio_service.lock().unwrap();
    service.channel().set_treble(treble);
}
