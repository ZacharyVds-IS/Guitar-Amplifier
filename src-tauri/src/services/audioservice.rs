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

pub fn push_input_samples(data: &[f32], producer: &mut HeapProd<f32>) {
    for &sample in data {
        let _ = producer.try_push(sample);
    }
}
pub fn fill_output_buffer(consumer: &mut HeapCons<f32>, output: &mut [f32]) {
    for sample in output.iter_mut() {
        *sample = consumer.try_pop().unwrap_or(0.0);
    }
}
fn handle_input(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut producer: HeapProd<f32>,
) -> cpal::Stream {
    device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            push_input_samples(data, &mut producer);
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
            // Inline debug buffer (not extracted, not tested)
            let mut debug_buf = [0.0f32; 10];
            for i in 0..10 {
                debug_buf[i] = consumer.try_pop().unwrap_or(0.0);
            }
            println!("Buffer: {:?}", debug_buf);

            // Pure, testable logic
            fill_output_buffer(&mut consumer, output);
        },
        move |err| eprintln!("Output error: {:?}", err),
        None,
    ).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::HeapRb;
    mod happy_path {
        use proptest::prelude::prop;
        use proptest::proptest;
        use super::*;

        proptest! {
            #[test]
            fn test_push_input_samples_arbitrary_data(input in prop::collection::vec(-1.0f32..1.0f32, 0..100)) {
                let rb = HeapRb::<f32>::new(1000);
                let (mut prod, mut cons) = rb.split();

                push_input_samples(&input, &mut prod);

                for &expected in &input {
                    assert_eq!(cons.try_pop(), Some(expected));
                }
            }
        }

        proptest! {
            #[test]
            fn test_fill_output_buffer(
                samples in prop::collection::vec(-1.0f32..1.0f32, 0..50),
                output_size in 1..100usize
            ) {
                let rb = HeapRb::<f32>::new(1000);
                let (mut prod, mut cons) = rb.split();

                for sample in &samples {
                    prod.try_push(*sample).unwrap();
                }

                let mut output = vec![0.0f32; output_size];
                fill_output_buffer(&mut cons, &mut output);

                for (i, &val) in output.iter().enumerate() {
                    if i < samples.len() {
                        assert_eq!(val, samples[i]);
                    } else {
                        assert_eq!(val, 0.0);
                    }
                }
            }
        }

        #[test]
        fn output_buffer_fills_with_zero_input_is_empty() {
            let rb = HeapRb::<f32>::new(4);
            let (_prod, mut cons) = rb.split();
            let mut out = [1.0f32; 4];

            fill_output_buffer(&mut cons, &mut out);

            assert_eq!(out, [0.0; 4]);
        }

    }

    mod failure_path {
        use super::*;

        #[test]
        fn buffer_overflow_drops_extra_samples_silently() {
            let rb = HeapRb::<f32>::new(4);
            let (mut prod, mut cons) = rb.split();
            let input = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];

            push_input_samples(&input, &mut prod);

            assert_eq!(cons.try_pop(), Some(10.0));
            assert_eq!(cons.try_pop(), Some(20.0));
            assert_eq!(cons.try_pop(), Some(30.0));
            assert_eq!(cons.try_pop(), Some(40.0));
            assert_eq!(cons.try_pop(), None);
        }
    }
}
