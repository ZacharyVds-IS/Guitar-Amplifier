use std::sync::Mutex;
use crate::domain::audio_device_dto::AudioDeviceDto;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;

#[tauri::command]
pub(crate) fn get_input_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_input_devices()
}
#[tauri::command]
pub(crate) fn get_output_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_output_devices()
}

#[tauri::command]
pub fn set_input_device(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device_id: String,
) -> Result<(), String> {
    let device = device_service
        .find_input_device_by_id(&device_id)
        .ok_or("Device not found")?;

    let mut audio = audio_service.lock().unwrap();
    audio.set_input_device(device);

    Ok(())
}

#[tauri::command]
pub fn set_output_device(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device_id: String,
) -> Result<(), String> {
    let device = device_service
        .find_output_device_by_id(&device_id)
        .ok_or("Device not found")?;

    let mut audio = audio_service.lock().unwrap();
    audio.set_output_device(device);

    Ok(())
}
