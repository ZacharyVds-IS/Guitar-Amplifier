use cpal::Host;
use cpal::traits::HostTrait;
use crate::domain::audio_device_dto::AudioDeviceDto;
use cpal::traits::DeviceTrait;

pub struct DeviceService {
    host:Host
}


impl DeviceService {

    pub fn new(host: Host) -> Self {
        Self { host }
    }

    pub fn get_input_devices(&self) -> Vec<AudioDeviceDto> {
        match self.host.input_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let desc = device.description().ok()?;
                    let name = desc.name().to_string();
                    let device_id = device.id().ok()?;
                    let id = format!("{:?}", device_id);
                    Some(AudioDeviceDto {
                        id,
                        name,
                    })
                })
                .collect(),
            Err(e) => {
                eprintln!("Failed to get input devices: {}", e);
                vec![]
            }
        }
    }

    pub fn get_output_devices(&self) -> Vec<AudioDeviceDto> {
        match self.host.output_devices() {
            Ok(devices) => devices
                .filter_map(|device| {
                    let desc = device.description().ok()?;
                    let name = desc.name().to_string();
                    let device_id = device.id().ok()?;
                    let id = format!("{:?}", device_id);

                    Some(AudioDeviceDto {
                        id,
                        name,
                    })
                })
                .collect(),
            Err(e) => {
                eprintln!("Failed to get output devices: {}", e);
                vec![]
            }
        }
    }


}