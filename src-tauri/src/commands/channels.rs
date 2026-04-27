use crate::domain::channel::Channel;
use crate::domain::channel_dto::ChannelDto;
use crate::services::audio_service::AudioService;
use std::sync::mpsc::channel;
use std::sync::Mutex;

#[tauri::command]
pub(crate) fn set_channel_index(
    audio_service: tauri::State<Mutex<AudioService>>,
    channel_index: usize,
) {
    let mut service = audio_service.inner().lock().unwrap();
    service.set_current_channel_index(channel_index);
}

#[tauri::command]
pub(crate) fn get_channel_index(audio_service: tauri::State<Mutex<AudioService>>) -> usize {
    let service = audio_service.inner().lock().unwrap();
    *service.current_channel_index()
}

#[tauri::command]
pub(crate) fn add_channel(audio_service: tauri::State<Mutex<AudioService>>, channel_name: String) {
    let mut service = audio_service.inner().lock().unwrap();
    service.add_channel(channel_name);
}

#[tauri::command]
pub(crate) fn get_all_channels(audio_service: tauri::State<Mutex<AudioService>>) -> Vec<ChannelDto> {
    let service = audio_service.inner().lock().unwrap();
    service
        .channels()
        .iter()
        .map(|channel| ChannelDto {
            name: channel.name().clone(),
            gain: channel.gain().load(std::sync::atomic::Ordering::Relaxed),
            tone_stack: crate::domain::tone_stack_dto::ToneStackDto {
                bass: channel
                    .tone_stack()
                    .bass()
                    .load(std::sync::atomic::Ordering::Relaxed),
                middle: channel
                    .tone_stack()
                    .middle()
                    .load(std::sync::atomic::Ordering::Relaxed),
                treble: channel
                    .tone_stack()
                    .treble()
                    .load(std::sync::atomic::Ordering::Relaxed),
            },
            volume: channel.volume().load(std::sync::atomic::Ordering::Relaxed),
        })
        .collect()
}
