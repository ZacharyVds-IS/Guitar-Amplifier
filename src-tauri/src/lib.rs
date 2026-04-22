pub mod commands;
pub mod services;
pub mod domain;
pub mod infrastructure;

use std::sync::Mutex;
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use tracing_subscriber::EnvFilter;
use crate::commands::default_controls::{set_gain, set_master_volume};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{get_input_device_list, get_output_device_list, set_input_device, set_output_device};
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //TODO remove temporary fields for changing in and output devices.
    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();
    let config = input.default_input_config().unwrap().config();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    tauri::Builder::default()
        .manage(Mutex::new(AudioService::new(input,output,config)))
        .manage(DeviceService::new(host))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_loopback, set_gain,get_input_device_list,get_output_device_list,set_input_device,set_output_device, set_master_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
