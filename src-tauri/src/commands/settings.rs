use std::sync::Mutex;
use crate::domain::audio_device_dto::AudioDeviceDto;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;

/// Retrieves a list of all available input devices.
///
/// Queries the [`DeviceService`] for all detected input devices and returns
/// them as [`AudioDeviceDto`] objects suitable for frontend display and selection.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// A [`Vec`] of [`AudioDeviceDto`] representing available input devices.
#[tauri::command]
pub(crate) fn get_input_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_input_devices()
}

/// Retrieves a list of all available output devices.
///
/// Queries the [`DeviceService`] for all detected output devices and returns
/// them as [`AudioDeviceDto`] objects suitable for frontend display and selection.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// A [`Vec`] of [`AudioDeviceDto`] representing available output devices.
#[tauri::command]
pub(crate) fn get_output_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_output_devices()
}

/// Switches the active input device.
///
/// Looks up the device by ID in the [`DeviceService`], then delegates to
/// [`AudioService::set_input_device`] to perform the hot-swap without interrupting
/// playback longer than necessary.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state for device lookup.
/// * `audio_service` - The shared [`AudioService`] state for performing the switch.
/// * `device_id` - The ID of the input device to activate.
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` if the device ID is not found
/// or the service state cannot be locked.
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

/// Switches the active output device.
///
/// Looks up the device by ID in the [`DeviceService`], then delegates to
/// [`AudioService::set_output_device`] to perform the hot-swap without interrupting
/// playback longer than necessary.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state for device lookup.
/// * `audio_service` - The shared [`AudioService`] state for performing the switch.
/// * `device_id` - The ID of the output device to activate.
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` if the device ID is not found
/// or the service state cannot be locked.
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


