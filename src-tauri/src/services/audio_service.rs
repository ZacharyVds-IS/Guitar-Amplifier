use std::sync::Arc;
use std::thread;
use cpal::{Device, StreamConfig};
use cpal::traits::StreamTrait;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};

pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
}

impl AudioService {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self {
            audio_handler: Arc::new(handler),
        }
    }

    //constructor for tests
    pub fn with_handler(handler: Arc<dyn AudioHandlerTrait>) -> Self {
        Self { audio_handler: handler }
    }

    pub fn start_loopback(&self) {
        let handler = self.audio_handler.clone();

        thread::spawn(move || {
            let (producer, consumer) = AudioHandler::create_ringbuffer(48000);

            let input_stream = handler.build_input_stream(producer);
            let output_stream = handler.build_output_stream(consumer);

            input_stream.play().unwrap();
            output_stream.play().unwrap();

            thread::park();
        });
    }
}
