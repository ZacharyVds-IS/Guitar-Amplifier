pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod services;

#[cfg(test)]
pub mod tests;

use crate::commands::channels::{
    add_channel, get_all_channels, get_channel_index, set_channel_index,
};
use crate::commands::default_controls::{
    get_amp_config, set_bass, set_gain, set_master_volume, set_middle, set_treble, set_volume,
    toggle_on_off,
};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{
    get_input_device_list, get_output_device_list, set_input_device, set_output_device,
};
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //TODO remove temporary fields for changing in and output devices.
    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();
    let config = input.default_input_config().unwrap().config();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .manage(Mutex::new(AudioService::new(input, output, config)))
        .manage(DeviceService::new(host))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_loopback,
            set_gain,
            get_input_device_list,
            get_output_device_list,
            set_input_device,
            set_output_device,
            set_master_volume,
            toggle_on_off,
            get_amp_config,
            set_bass,
            set_middle,
            set_treble,
            set_volume,
            set_channel_index,
            get_channel_index,
            add_channel,
            get_all_channels
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
