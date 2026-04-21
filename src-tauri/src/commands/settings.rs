use crate::domain::audio_device_dto::AudioDeviceDto;
use crate::services::device_service::DeviceService;

#[tauri::command]
pub(crate) fn get_input_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_input_devices()
}
#[tauri::command]
pub(crate) fn get_output_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_output_devices()
}

