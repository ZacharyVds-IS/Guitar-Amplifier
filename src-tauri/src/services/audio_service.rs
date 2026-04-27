use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel::Channel;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use atomic_float::AtomicF32;
use cpal::{Device, StreamConfig};
use derive_getters::Getters;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, info};

/// The main service that orchestrates real-time audio loopback between an input and output device.
///
/// `AudioService` manages the lifecycle of an audio processing pipeline, including:
/// - Starting and stopping the loopback thread
/// - Routing audio samples through the [`Channel`] processing chain (gain, master volume)
/// - Hot-swapping input/output devices without requiring a full restart
#[derive(Getters)]
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    channel: Channel,
    master_volume: Arc<AtomicF32>,
}

impl AudioService {
    /// Creates a new `AudioService` using the provided CPAL input/output devices and stream config.
    ///
    /// An [`AudioHandler`] is constructed internally from the given parameters.
    ///
    /// # Arguments
    ///
    /// * `input_device` - The CPAL device to capture audio from.
    /// * `output_device` - The CPAL device to send processed audio to.
    /// * `config` - The shared [`StreamConfig`] applied to both streams.
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self::new_with_handler(Arc::new(handler))
    }

    /// Creates an `AudioService` with a custom handler.
    ///
    /// This constructor is primarily intended for unit and integration testing,
    /// where a mock [`AudioHandlerTrait`] implementation can be injected in place
    /// of a real [`AudioHandler`].
    ///
    /// # Arguments
    ///
    /// * `handler` - An [`Arc`]-wrapped implementation of [`AudioHandlerTrait`].
    pub fn new_with_handler(handler: Arc<dyn AudioHandlerTrait>) -> Self {
        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
            channel: Channel::new("Main".to_string(), None, None),
            master_volume: Arc::new(AtomicF32::new(1.0)),
        }
    }

    /// Starts the audio loopback on a dedicated background thread.
    ///
    /// Audio samples are read from the input stream, passed through the gain and
    /// master volume processors defined on the [`Channel`], and written to the
    /// output stream via lock-free ring buffers.
    ///
    /// If the loopback is already active this method is a no-op.
    pub fn start_loopback(&mut self) {
        if self.is_active {
            return;
        }

        info!("Starting audio loopback");
        self.is_active = true;

        let handler = self.audio_handler.clone();
        let channel = self.channel.clone(); // shared Arc<AtomicF32>
        let master_volume_arc = self.master_volume.clone();

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
                let mut volume = GainProcessor::new(channel.volume());
                let mut master_volume = GainProcessor::new(master_volume_arc);
                let mut tone_stack = ToneStackProcessor::new(channel.tone_stack());

                loop {
                    if worker_shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    if let Some(sample) = i_consumer.try_pop() {
                        let gain_sample = gain.process(sample);

                        let eq_sample = tone_stack.process(gain_sample);

                        //EFFECTS GO HERE

                        let wet_sample= volume.process(eq_sample);

                        //for debugging: print the tone stack values
                        //tone_stack.print_tone_stack(eq_sample, &mut fft_buffer, FFT_SIZE);

                        let processed = master_volume.process(wet_sample);
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

    /// Stops the audio loopback and joins the background thread.
    ///
    /// Unparks the loopback thread, signals the inner worker to shut down,
    /// and waits for both threads to finish. If the loopback is not currently
    /// active this method is a no-op.
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

    /// Sets the master volume value.
    ///
    /// The master volume value is atomically updated and will be read by the audio processing
    /// thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `master_volume` - The new master volume value. Must be positive (> 0.0).
    ///
    /// # Panics
    ///
    /// Panics if `master_volume` is negative or zero.
    pub fn set_master_volume(&self, master_volume: f32) {
        if master_volume.is_sign_positive() {
            self.master_volume.store(master_volume, Ordering::Relaxed);
        } else {
            error!("Master volume must be a positive number");
            panic!("Master volume must be positive");
        }
    }

    /// Replaces the underlying audio handler, restarting the loopback if it was running.
    ///
    /// If the loopback is active when this method is called it will be stopped,
    /// the handler swapped, and then the loopback restarted automatically.
    ///
    /// # Arguments
    ///
    /// * `new_handler` - The replacement [`AudioHandlerTrait`] implementation.
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

    /// Switches the audio input device without interrupting playback longer than necessary.
    ///
    /// Constructs a new [`AudioHandler`] that pairs the given `input` device with the
    /// existing output device and stream config, then delegates to [`set_audio_handler`].
    ///
    /// # Arguments
    ///
    /// * `input` - The new CPAL input device to capture audio from.
    ///
    /// [`set_audio_handler`]: AudioService::set_audio_handler
    pub fn set_input_device(&mut self, input: Device) {
        info!("Switching input device");

        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(input, old.output_device().clone(), old.config().clone());

        self.set_audio_handler(Arc::new(new_handler));
    }

    /// Switches the audio output device without interrupting playback longer than necessary.
    ///
    /// Constructs a new [`AudioHandler`] that pairs the existing input device with the
    /// given `output` device and stream config, then delegates to [`set_audio_handler`].
    ///
    /// # Arguments
    ///
    /// * `output` - The new CPAL output device to send processed audio to.
    ///
    /// [`set_audio_handler`]: AudioService::set_audio_handler
    pub fn set_output_device(&mut self, output: Device) {
        info!("Switching output device");

        let old = self.audio_handler.clone();
        let new_handler =
            AudioHandler::new(old.input_device().clone(), output, old.config().clone());

        self.set_audio_handler(Arc::new(new_handler));
    }

    /// Toggles the audio loopback on or off.
    ///
    /// - If `is_on` is `true` and the loopback is not active, [`start_loopback`] is called.
    /// - If `is_on` is `false` and the loopback is active, [`stop_loopback`] is called.
    /// - If the requested state already matches the current state, this method is a no-op.
    ///
    /// [`start_loopback`]: AudioService::start_loopback
    /// [`stop_loopback`]: AudioService::stop_loopback
    pub fn toggle_loopback(&mut self, is_on: bool) {
        if self.is_active == is_on {
            return;
        }
        if is_on == false {
            self.stop_loopback();
        } else {
            self.start_loopback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn master_volume_set_to_positive_value_should_succeed() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock));
            service.set_master_volume(0.5);
            assert_eq!(service.master_volume().load(Ordering::Relaxed), 0.5);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Master volume must be positive")]
        fn master_volume_set_to_negative_value_should_panic() {
            let mock = MockAudioHandlerTrait::new();
            let service = AudioService::new_with_handler(Arc::new(mock));
            service.set_master_volume(-0.5);
        }
    }
}