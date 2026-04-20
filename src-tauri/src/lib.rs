use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;
use ringbuf::consumer::Consumer;
use ringbuf::HeapRb;
use ringbuf::producer::Producer;
use ringbuf::traits::Split;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_loopback() {
    thread::spawn(|| {
        let host = cpal::default_host();

        let input_device = host.default_input_device().expect("No input device");
        let output_device = host.default_output_device().expect("No output device");

        let input_config = input_device.default_input_config().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        println!("Input:  {}", input_device.id().unwrap());
        println!("Output: {}", output_device.id().unwrap());

        // Heap-allocated ring buffer, SPSC, perfect for CPAL callbacks
        let rb = HeapRb::<f32>::new(48000);
        let (mut producer, mut consumer) = rb.split();

        // Input: mic → producer
        let input_stream = input_device
            .build_input_stream(
                &input_config.into(),
                move |data: &[f32], _| {
                    for &sample in data {
                        let _ = producer.try_push(sample);
                    }
                },
                move |err| eprintln!("Input error: {:?}", err),
                None,
            )
            .unwrap();

        // Output: consumer → speakers
        let output_stream = output_device
            .build_output_stream(
                &output_config.into(),
                move |output: &mut [f32], _| {
                    /*
                    // For quiet debugging: print output samples
                    for sample in output.iter_mut() {
                        println!("Output sample: {}", sample);
                    }
                     */
                    for sample in output.iter_mut() {
                        *sample = consumer.try_pop().unwrap_or(0.0);
                    }
                },
                move |err| eprintln!("Output error: {:?}", err),
                None,
            )
            .unwrap();
        input_stream.play().unwrap();
        output_stream.play().unwrap();
        println!("Loopback running…");
        // Hold the thread alive to keep the streams alive without extra CPU usage
        thread::park();
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_loopback])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
