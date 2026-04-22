use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel::Channel;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::gain_processor::GainProcessor;
use cpal::{Device, StreamConfig};
use derive_getters::Getters;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use tracing::info;
use crate::services::tone_stack::tone_stack_processor::ToneStackProcessor;

#[derive(Getters)]
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    channel: Channel,
}

impl AudioService {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self::new_with_handler(Arc::new(handler))
    }

    /// Creates an `AudioService` with a custom handler. Useful for testing with mock handlers.
    pub fn new_with_handler(handler: Arc<dyn AudioHandlerTrait>) -> Self {
        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
            channel: Channel::new("Main".to_string(), None, None),
        }
    }

    pub fn start_loopback(&mut self) {
        if self.is_active {
            return;
        }

        info!("Starting audio loopback");
        self.is_active = true;

        let handler = self.audio_handler.clone();
        let channel = self.channel.clone(); // shared Arc<AtomicF32>

        let thread = thread::spawn(move || {
            const FFT_SIZE: usize = 2048;
            let mut fft_buffer: Vec<f32> = Vec::with_capacity(FFT_SIZE);

            let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(48000);
            let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(48000);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            let shutdown = Arc::new(AtomicBool::new(false));
            let worker_shutdown = shutdown.clone();

            let worker = thread::spawn(move || {
                let mut gain = GainProcessor::new(channel.gain());
                let mut master_volume = GainProcessor::new(channel.master_volume());
                let mut tone_stack = ToneStackProcessor::new(channel.tone_stack());

                loop {
                    if worker_shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    if let Some(sample) = i_consumer.try_pop() {
                        let gain_sample = gain.process(sample);

                        let eq_sample = tone_stack.process(gain_sample);

                        //for debugging: print the tone stack values
                        tone_stack.print_tone_stack(eq_sample, &mut fft_buffer, FFT_SIZE);

                        let processed = master_volume.process(eq_sample);
                        let _ = o_producer.try_push(processed);
                    } else {
                        thread::yield_now();
                    }
                }
            });

            input_stream.play();
            output_stream.play();

            thread::park();

            shutdown.store(true, Ordering::SeqCst);
            let _ = worker.join();
        });

        self.loopback_thread = Some(thread);
    }

    pub fn stop_loopback(&mut self) {
        if !self.is_active {
            return;
        }

        info!("Stopping audio loopback");

        if let Some(handle) = self.loopback_thread.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }

        self.is_active = false;
    }

    pub(crate) fn set_audio_handler(&mut self, new_handler: Arc<dyn AudioHandlerTrait>) {
        let was_active = self.is_active;
        if was_active {
            self.stop_loopback();
        }

        self.audio_handler = new_handler;

        if was_active {
            self.start_loopback();
        }
    }

    pub fn set_input_device(&mut self, input: Device) {
        info!("Switching input device");

        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(input, old.output_device().clone(), old.config().clone());

        self.set_audio_handler(Arc::new(new_handler));
    }

    pub fn set_output_device(&mut self, output: Device) {
        info!("Switching output device");

        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(old.input_device().clone(), output, old.config().clone());

        self.set_audio_handler(Arc::new(new_handler));
    }

    pub fn toggle_loopback(&mut self, is_on: bool){
        if self.is_active == is_on{
            return;
        }
        if is_on == false{
            self.stop_loopback();
            return;
        }
        self.start_loopback();

    }
}
