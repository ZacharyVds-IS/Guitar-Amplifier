pub mod commands;
pub mod services;
pub mod domain;
pub mod infrastructure;

use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use crate::commands::default_controls::{set_gain, set_master_volume};
use crate::commands::loopback::start_loopback;
use crate::services::audio_service::AudioService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //TODO remove temporary fields for changing in and output devices.
    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();
    let config = input.default_input_config().unwrap().config();

    tauri::Builder::default()
        .manage(AudioService::new(input,output,config))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_loopback, set_gain, set_master_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
