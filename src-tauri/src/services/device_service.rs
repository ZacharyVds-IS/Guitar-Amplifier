use crate::domain::dto::audio_device_dto::AudioDeviceDto;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::{available_hosts, default_host, host_from_id, Device, Host};
use std::sync::Mutex;
use tracing::error;

const AUDIO_DRIVER_DEFAULT: &str = "Default";
const AUDIO_DRIVER_ASIO: &str = "ASIO";

/// Service for managing audio device enumeration and lookup.
///
/// `DeviceService` wraps a CPAL [`Host`] and provides convenient methods to:
/// - List available input and output devices
/// - Look up devices by their ID
/// - Convert device information into [`AudioDeviceDto`] for frontend consumption
#[derive(Default)]
pub struct DeviceService {
    selected_audio_driver: Mutex<String>,
}

impl DeviceService {
    /// Creates a new `DeviceService` with the given CPAL host.
    ///
    /// # Arguments
    ///
    /// * `host` - A CPAL [`Host`] instance.
    pub fn new() -> Self {
        Self {
            selected_audio_driver: Mutex::new(AUDIO_DRIVER_DEFAULT.to_string()),
        }
    }

    pub fn available_audio_drivers(&self) -> Vec<String> {
        if cfg!(target_os = "windows") {
            vec![
                AUDIO_DRIVER_DEFAULT.to_string(),
                AUDIO_DRIVER_ASIO.to_string(),
            ]
        } else {
            vec![AUDIO_DRIVER_DEFAULT.to_string()]
        }
    }

    pub fn selected_audio_driver(&self) -> String {
        self.selected_audio_driver
            .lock()
            .map(|driver| driver.clone())
            .unwrap_or_else(|_| AUDIO_DRIVER_DEFAULT.to_string())
    }

    pub fn set_selected_audio_driver(&self, driver: &str) -> Result<(), String> {
        let normalized = Self::normalize_driver(driver)
            .ok_or_else(|| format!("Unsupported audio driver '{}'.", driver))?;

        let mut selected = self
            .selected_audio_driver
            .lock()
            .map_err(|_| "Failed to lock selected audio driver".to_string())?;
        *selected = normalized.to_string();
        Ok(())
    }

    pub fn is_asio_selected(&self) -> bool {
        cfg!(target_os = "windows") && self.selected_audio_driver() == AUDIO_DRIVER_ASIO
    }

    fn normalize_driver(driver: &str) -> Option<&'static str> {
        if driver.eq_ignore_ascii_case(AUDIO_DRIVER_DEFAULT) {
            return Some(AUDIO_DRIVER_DEFAULT);
        }

        if cfg!(target_os = "windows") && driver.eq_ignore_ascii_case(AUDIO_DRIVER_ASIO) {
            return Some(AUDIO_DRIVER_ASIO);
        }

        None
    }

    fn host_for_selected_driver(&self) -> Result<Host, String> {
        Self::host_for_driver(&self.selected_audio_driver())
    }

    fn host_for_driver(driver: &str) -> Result<Host, String> {
        if cfg!(target_os = "windows") {
            if driver.eq_ignore_ascii_case(AUDIO_DRIVER_ASIO) {
                return Self::host_from_backend_name("Asio");
            }

            return Self::host_from_backend_name("Wasapi").or_else(|_| Ok(default_host()));
        }

        Ok(default_host())
    }

    fn host_from_backend_name(backend_name: &str) -> Result<Host, String> {
        for host_id in available_hosts() {
            if format!("{:?}", host_id).eq_ignore_ascii_case(backend_name) {
                return host_from_id(host_id)
                    .map_err(|e| format!("Failed to initialize {} host: {}", backend_name, e));
            }
        }

        Err(format!("{} host is not available", backend_name))
    }

    fn device_to_audio_device_dto(device: Device, sample_rate: u32) -> Option<AudioDeviceDto> {
        let desc = device.description().ok()?;
        let name = desc.name().to_string();
        let device_id = device.id().ok()?;
        let id = format!("{:?}", device_id);

        Some(AudioDeviceDto {
            id,
            name,
            sample_rate,
        })
    }

    fn get_duplex_devices(&self) -> Vec<AudioDeviceDto> {
        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for duplex device listing: {}", e);
                return vec![];
            }
        };

        match host.devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let input_config = device.default_input_config().ok()?;
                    let output_config = device.default_output_config().ok()?;
                    let sample_rate = input_config.sample_rate().min(output_config.sample_rate());
                    Self::device_to_audio_device_dto(device, sample_rate)
                })
                .collect(),
            Err(e) => {
                error!("Failed to get duplex devices: {}", e);
                vec![]
            }
        }
    }

    pub fn default_devices_for_selected_driver(&self) -> Result<(Device, Device), String> {
        let host = self.host_for_selected_driver()?;

        if self.is_asio_selected() {
            let device = host
                .devices()
                .map_err(|e| format!("Failed to enumerate ASIO devices: {}", e))?
                .find(|device| {
                    device.default_input_config().is_ok() && device.default_output_config().is_ok()
                })
                .ok_or_else(|| {
                    "No ASIO device with both input and output support was found".to_string()
                })?;

            return Ok((device.clone(), device));
        }

        let input = host
            .default_input_device()
            .ok_or_else(|| "No default input device found".to_string())?;
        let output = host
            .default_output_device()
            .ok_or_else(|| "No default output device found".to_string())?;
        Ok((input, output))
    }

    /// Retrieves a list of all available input devices.
    ///
    /// Queries the CPAL host for input devices, converts them to [`AudioDeviceDto`],
    /// and returns them. If device enumeration fails, an empty list is returned
    /// and an error is added to the logs.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`AudioDeviceDto`] representing available input devices.
    pub fn get_input_devices(&self) -> Vec<AudioDeviceDto> {
        if self.is_asio_selected() {
            return self.get_duplex_devices();
        }

        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for input device listing: {}", e);
                return vec![];
            }
        };

        match host.input_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let device_config = device.default_input_config().ok()?;
                    Self::device_to_audio_device_dto(device, device_config.sample_rate())
                })
                .collect(),
            Err(e) => {
                error!("Failed to get input devices: {}", e);
                vec![]
            }
        }
    }

    /// Retrieves a list of all available output devices.
    ///
    /// Queries the CPAL host for output devices, converts them to [`AudioDeviceDto`],
    /// and returns them. If device enumeration fails, an empty list is returned
    /// and an error is printed to stderr.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`AudioDeviceDto`] representing available output devices.
    pub fn get_output_devices(&self) -> Vec<AudioDeviceDto> {
        if self.is_asio_selected() {
            return self.get_duplex_devices();
        }

        let host = match self.host_for_selected_driver() {
            Ok(host) => host,
            Err(e) => {
                error!("Failed to initialize host for output device listing: {}", e);
                return vec![];
            }
        };

        match host.output_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let device_config = device.default_output_config().ok()?;
                    Self::device_to_audio_device_dto(device, device_config.sample_rate())
                })
                .collect(),
            Err(e) => {
                error!("Failed to get output devices: {}", e);
                vec![]
            }
        }
    }

    /// Finds an input device by its string ID.
    ///
    /// Searches through the host's input devices for one whose debug-formatted
    /// ID matches the given string.
    ///
    /// # Arguments
    ///
    /// * `id` - The device ID string to search for (debug-formatted CPAL device ID).
    ///
    /// # Returns
    ///
    /// `Some(device)` if a matching input device is found, `None` otherwise.
    pub fn find_input_device_by_id(&self, id: &str) -> Option<cpal::Device> {
        if self.is_asio_selected() {
            return self.find_duplex_device_by_id(id);
        }

        let host = self.host_for_selected_driver().ok()?;
        let devices = host.input_devices().ok()?;

        for device in devices {
            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }

    /// Finds an output device by its string ID.
    ///
    /// Searches through the host's output devices for one whose debug-formatted
    /// ID matches the given string.
    ///
    /// # Arguments
    ///
    /// * `id` - The device ID string to search for (debug-formatted CPAL device ID).
    ///
    /// # Returns
    ///
    /// `Some(device)` if a matching output device is found, `None` otherwise.
    pub fn find_output_device_by_id(&self, id: &str) -> Option<cpal::Device> {
        if self.is_asio_selected() {
            return self.find_duplex_device_by_id(id);
        }

        let host = self.host_for_selected_driver().ok()?;
        let devices = host.output_devices().ok()?;

        for device in devices {
            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }

    fn find_duplex_device_by_id(&self, id: &str) -> Option<cpal::Device> {
        let host = self.host_for_selected_driver().ok()?;
        let devices = host.devices().ok()?;

        for device in devices {
            if device.default_input_config().is_err() || device.default_output_config().is_err() {
                continue;
            }

            let device_id = device.id().ok()?;
            if format!("{:?}", device_id) == id {
                return Some(device);
            }
        }

        None
    }
}
