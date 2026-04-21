use std::sync::Arc;
use std::thread;
use cpal::{Device, StreamConfig};
use cpal::traits::StreamTrait;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};

#[derive(Getters)]
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    channel: Channel
}

impl AudioService {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self {
            audio_handler: Arc::new(handler),
            loopback_thread: None,
            is_active: false,
             channel: Channel::new("Main".to_string(), 1.0),
        }
    }


    ///Start loopback creates a new thread used for audio processing and keeps it alive until stopped by stop_loopback.
    pub fn start_loopback(&mut self) {
        info!("Starting audio loopback");
        self.is_active = true;
        let handler = self.audio_handler.clone();
        let channel = self.channel.clone();

        thread::spawn(move || {
            let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(48000);
            let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(48000);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            thread::spawn(move || {
                let mut gain = GainProcessor::new(channel.gain_handle());

                loop {
                    if let Some(sample) = i_consumer.try_pop() {
                        let processed = gain.process(sample);
                        let _ = o_producer.try_push(processed);
                    } else {
                        std::thread::yield_now();
                    }
                }
            });

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
