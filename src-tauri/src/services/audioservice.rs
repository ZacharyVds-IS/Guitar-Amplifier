use std::thread;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, traits::Split, HeapProd, HeapCons};
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;

pub fn start_loopback() {
    thread::spawn(|| {
        let host = cpal::default_host();

        let input_device = host.default_input_device().expect("No input device");
        let output_device = host.default_output_device().expect("No output device");

        let input_config = input_device.default_input_config().unwrap();
        let output_config = output_device.default_output_config().unwrap();

        println!("Input:  {}", input_device.id().unwrap());
        println!("Output: {}", output_device.id().unwrap());

        let rb = HeapRb::<f32>::new(48000);
        let (producer, consumer) = rb.split();

        let input_stream = handle_input(&input_device, input_config.into(), producer);
        let output_stream = process_and_output(&output_device, output_config.into(), consumer);

        input_stream.play().unwrap();
        output_stream.play().unwrap();

        println!("Loopback running…");
        thread::park();
    });
}

fn handle_input(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut producer: HeapProd<f32>,
) -> cpal::Stream {
    device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            for &sample in data {
                let _ = producer.try_push(sample);
            }
        },
        move |err| eprintln!("Input error: {:?}", err),
        None,
    ).unwrap()
}

fn process_and_output(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut consumer: HeapCons<f32>,
) -> cpal::Stream {
    device.build_output_stream(
        &config,
        move |output: &mut [f32], _| {
            let mut debug_buf = [0.0f32; 10];
            for i in 0..10 {
                debug_buf[i] = consumer.try_pop().unwrap_or(0.0);
            }
            println!("Buffer: {:?}", debug_buf);

            // Output audio
            for sample in output.iter_mut() {
                *sample = consumer.try_pop().unwrap_or(0.0);
            }
        },
        move |err| eprintln!("Output error: {:?}", err),
        None,
    ).unwrap()
}
