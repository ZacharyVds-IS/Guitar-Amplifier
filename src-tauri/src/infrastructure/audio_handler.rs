use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use ringbuf::traits::Split;
use ringbuf::{HeapCons, HeapProd, HeapRb};
use derive_getters::Getters;
use mockall::automock;

pub trait PlayableStream: Send {
    fn play(&self);
}

impl PlayableStream for Stream {
    fn play(&self) {
        StreamTrait::play(self).unwrap();
    }
}

#[automock]
pub trait AudioHandlerTrait: Send + Sync {
    fn build_input_stream(&self, prod: HeapProd<f32>) -> Box<dyn PlayableStream>;
    fn build_output_stream(&self, cons: HeapCons<f32>) -> Box<dyn PlayableStream>;
    fn input_device(&self) -> &Device;
    fn output_device(&self) -> &Device;
    fn config(&self) -> &StreamConfig;
}

#[derive(Clone, Getters)]
pub struct AudioHandler {
    input_device: Device,
    output_device: Device,
    config: StreamConfig,
}
impl AudioHandler {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        Self {
            input_device,
            output_device,
            config,
        }
    }


    pub fn create_ringbuffer(size: usize) -> (HeapProd<f32>, HeapCons<f32>) {
        let rb = HeapRb::<f32>::new(size);
        rb.split()
    }

    pub fn set_output_device(&mut self, output_device: Device) {
        self.output_device = output_device;
    }

    pub fn set_input_device(&mut self, input_device: Device) {
        self.input_device = input_device;
    }
}

impl AudioHandlerTrait for AudioHandler {
    fn build_input_stream(&self, mut producer: HeapProd<f32>) -> Box<dyn PlayableStream> {
        let stream = self.input_device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _| {
                    for &s in data {
                        let _ = producer.try_push(s);
                    }
                },
                move |err| eprintln!("Input error: {:?}", err),
                None,
            )
            .unwrap();
        Box::new(stream)
    }

    fn build_output_stream(&self, mut consumer: HeapCons<f32>) -> Box<dyn PlayableStream> {
        let stream = self.output_device
            .build_output_stream(
                &self.config,
                move |out: &mut [f32], _| {
                    //println!("Output buffer: {:?}", &out[..10.min(out.len())]);
                    for o in out.iter_mut() {
                        *o = consumer.try_pop().unwrap_or(0.0);
                    }
                },
                move |err| eprintln!("Output error: {:?}", err),
                None,
            )
            .unwrap();
        Box::new(stream)
    }

    fn input_device(&self) -> &Device {
        &self.input_device
    }

    fn output_device(&self) -> &Device {
        &self.output_device
    }

    fn config(&self) -> &StreamConfig {
        &self.config
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn test_input_callback_pushes_samples() {
            let (mut prod, mut cons) = AudioHandler::create_ringbuffer(16);

            let input_data = vec![0.1, 0.2, 0.3, 0.4];
            {
                for &s in &input_data {
                    let _ = prod.try_push(s);
                }
            }

            for expected in input_data {
                assert_eq!(cons.try_pop(), Some(expected));
            }
        }
        #[test]
        fn test_output_callback_reads_samples() {
            let (mut prod, mut cons) = AudioHandler::create_ringbuffer(16);
            prod.try_push(10.0).unwrap();
            prod.try_push(20.0).unwrap();
            prod.try_push(30.0).unwrap();

            let mut out = [0.0f32; 5];
            {
                for o in out.iter_mut() {
                    *o = cons.try_pop().unwrap_or(0.0);
                }
            }

            assert_eq!(out, [10.0, 20.0, 30.0, 0.0, 0.0]);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn test_input_callback_drops_when_full() {
            let (mut prod, mut cons) = AudioHandler::create_ringbuffer(3);

            let input_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

            for &s in &input_data {
                let _ = prod.try_push(s);
            }

            assert_eq!(cons.try_pop(), Some(1.0));
            assert_eq!(cons.try_pop(), Some(2.0));
            assert_eq!(cons.try_pop(), Some(3.0));
            assert_eq!(cons.try_pop(), None);
        }

        #[test]
        fn test_output_callback_zero_fills_when_empty() {
            let (_prod, mut cons) = AudioHandler::create_ringbuffer(8);

            let mut out = [1.0f32; 4];

            {
                for o in out.iter_mut() {
                    *o = cons.try_pop().unwrap_or(0.0);
                }
            }

            assert_eq!(out, [0.0, 0.0, 0.0, 0.0]);
        }
    }
}
