use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use cpal::traits::StreamTrait;
use cpal::{Device, StreamConfig};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::info;

pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
}

impl AudioService {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self {
            audio_handler: Arc::new(handler),
            loopback_thread: None,
            is_active: false,
        }
    }

    //constructor for tests
    pub fn with_handler(handler: Arc<dyn AudioHandlerTrait>) -> Self {
        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
        }
    }

    ///Start loopback creates a new thread used for audio processing and keeps it alive until stopped by stop_loopback.
    pub fn start_loopback(&mut self) {
        info!("Starting audio loopback");
        self.is_active = true;
        let handler = self.audio_handler.clone();

        let thread = thread::spawn(move || {
            let (producer, consumer) = AudioHandler::create_ringbuffer(48000);

            let input_stream = handler.build_input_stream(producer);
            let output_stream = handler.build_output_stream(consumer);

            input_stream.play().unwrap();
            output_stream.play().unwrap();

            thread::park();
        });

        self.loopback_thread = Some(thread);
    }

    ///Stop loopback unparks the active thread and stops the loopback thread ready for re-creation.
    pub fn stop_loopback(&mut self) {
        info!("Stopping audio loopback");
        if let Some(handle) = self.loopback_thread.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
        self.is_active = false;
    }

    pub fn set_input_device(&mut self, input: Device) {
        info!("Switching input device");
        let was_active = self.is_active.clone();
        if was_active {
            self.stop_loopback();
        };
        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(input, old.output_device().clone(), old.config().clone());
        self.audio_handler = Arc::new(new_handler);
        if was_active {
            self.start_loopback();
        }
    }

    pub fn set_output_device(&mut self, output: Device) {
        info!("Switching output device");
        let was_active = self.is_active.clone();
        if was_active {
            self.stop_loopback();
        };
        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(old.input_device().clone(), output, old.config().clone());
        self.audio_handler = Arc::new(new_handler);
        if was_active {
            self.start_loopback();
        }
    }
}
