use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel::Channel;
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::resampler::resampler::ResamplePolicy;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use cpal::{BufferSize, Device, StreamConfig};
use derive_getters::Getters;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tracing::info;


/// The main service that orchestrates real-time audio loopback between an input and output device.
///
/// `AudioService` manages the full lifecycle of the audio processing pipeline:
///
/// - **Device management** — holds references to the active CPAL input/output devices
///   through an [`AudioHandlerTrait`] implementation and supports hot-swapping either
///   device without a full restart.
/// - **Resampling** — on `start_loopback` the input and output sample rates are compared
///   and a [`ResamplePolicy`] is selected automatically:
///   - `input == output` → no resampling, zero overhead.
///   - `input > output` → downsample before the DSP chain.
///   - `input < output` → upsample after the DSP chain.
/// - **DSP chain** — every sample passes through gain, tone stack, and master volume
///   processors in order. Additional processors can be inserted into `start_loopback`'s
///   `run_dsp` closure.
/// - **Thread lifecycle** — the loopback runs on a dedicated background thread with a
///   lock-free ring buffer between the CPAL callbacks and the worker; the thread is
///   cleanly shut down via [`stop_loopback`].
///
/// [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
/// [`ResamplePolicy`]: crate::services::processors::resampler::resampler::ResamplePolicy
/// [`stop_loopback`]: AudioService::stop_loopback
#[derive(Getters)]
pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    loopback_thread: Option<JoinHandle<()>>,
    is_active: bool,
    channel: Channel,
    preferred_buffer_frames: u32,
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
    /// * `input_config` - The [`StreamConfig`] used for the input stream.
    /// * `output_config` - The [`StreamConfig`] used for the output stream.
    pub fn new(
        input_device: Device,
        output_device: Device,
        input_config: StreamConfig,
        output_config: StreamConfig,
    ) -> Self {
        let handler = AudioHandler::new(input_device, output_device, input_config, output_config);
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
        let preferred_buffer_frames = match handler.output_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => 256,
        };

        Self {
            audio_handler: handler,
            loopback_thread: None,
            is_active: false,
            channel: Channel::new("Main".to_string(), None, None),
            preferred_buffer_frames,
        }
    }

    /// Current user-selected stream buffer size in frames.
    pub fn buffer_size_frames(&self) -> u32 {
        self.preferred_buffer_frames
    }

    /// Applies a user-selected buffer size to both input and output streams.
    ///
    /// The service rebuilds the underlying handler using the currently selected devices
    /// and restarts loopback automatically if it was active.
    pub fn set_buffer_size_frames(&mut self, frames: u32) -> Result<(), String> {
        if !(64..=4096).contains(&frames) {
            return Err("Buffer size must be between 64 and 4096 frames".to_string());
        }

        let previous_frames = self.preferred_buffer_frames;
        if previous_frames == frames {
            return Ok(());
        }

        info!(
            previous_buffer_size_frames = previous_frames,
            new_buffer_size_frames = frames,
            "Updating audio stream buffer size"
        );

        let old = self.audio_handler.clone();
        let mut input_config = old.input_config().clone();
        let mut output_config = old.output_config().clone();
        input_config.buffer_size = BufferSize::Fixed(frames);
        output_config.buffer_size = BufferSize::Fixed(frames);

        let new_handler = AudioHandler::new(
            old.input_device().clone(),
            old.output_device().clone(),
            input_config,
            output_config,
        );

        self.preferred_buffer_frames = frames;
        self.set_audio_handler(Arc::new(new_handler));
        Ok(())
    }

    /// Starts the audio loopback on a dedicated background thread.
    ///
    /// On startup the service:
    /// 1. Reads the input and output sample rates from the active [`AudioHandlerTrait`].
    /// 2. Selects a [`ResamplePolicy`] based on those rates (logged at `info` level).
    /// 3. Builds a pair of lock-free ring buffers sized from the configured preferred buffer size.
    /// 4. Asks the handler to open the input and output CPAL streams.
    /// 5. Spawns a worker thread that:
    ///    - Pops samples from the input buffer.
    ///    - Routes them through the [`ResamplePolicy`] which interleaves `run_dsp` at
    ///      the correct point (before or after resampling).
    ///    - Pushes results into the output buffer.
    ///    - On shutdown, flushes any remaining resampler tail before exiting.
    ///
    /// If the loopback is already active this method is a no-op.
    ///
    /// [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
    /// [`ResamplePolicy`]: crate::services::processors::resampler::resampler::ResamplePolicy
    pub fn start_loopback(&mut self) {
        if self.is_active { return; }
        self.is_active = true;

        let handler = self.audio_handler.clone();
        let channel = self.channel.clone();
        let ringbuffer_size = ((self.preferred_buffer_frames as usize) * 4).max(512);

        let thread = thread::spawn(move || {
            const RESAMPLER_CHUNK_SIZE: usize = 256;

            let mut policy = ResamplePolicy::from_rates(
                handler.input_sample_rate(),
                handler.output_sample_rate(),
                RESAMPLER_CHUNK_SIZE,
            );

            let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
            let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            let shutdown = Arc::new(AtomicBool::new(false));
            let worker_shutdown = shutdown.clone();

            let worker = thread::spawn(move || {
                let mut gain = GainProcessor::new(channel.gain());
                let mut tone_stack = ToneStackProcessor::new(channel.tone_stack());
                let mut master_volume = GainProcessor::new(channel.master_volume());

                let mut run_dsp = |sample: f32| -> f32 {
                    let s = gain.process(sample);
                    let s = tone_stack.process(s);
                    master_volume.process(s)
                };

                // --- DEBUG COUNTERS ---
                let mut samples_received_total: u64 = 0;
                let mut peak_volume: f32 = 0.0;
                let mut last_log_time = Instant::now();

                loop {
                    if worker_shutdown.load(Ordering::SeqCst) { break; }

                    if let Some(sample) = i_consumer.try_pop() {
                        samples_received_total += 1;
                        let abs_s = sample.abs();
                        if abs_s > peak_volume { peak_volume = abs_s; }

                        if last_log_time.elapsed() >= Duration::from_secs(2) {
                            println!("[LOOPBACK] Worker alive. Samples: {}, Peak: {:.4}", samples_received_total, peak_volume);
                            peak_volume = 0.0;
                            last_log_time = Instant::now();
                        }

                        // Run DSP through the resample policy (handles up/down-sampling transparently).
                        for dsp_sample in policy.process(sample, &mut |s| run_dsp(s)) {
                            let _ = o_producer.try_push(dsp_sample);
                        }
                    } else {
                        thread::yield_now();
                    }
                }
            });

            println!("[LOOPBACK] Starting CPAL streams...");
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
    pub fn set_input_device(&mut self, input: Device, input_config: StreamConfig) {
        info!("Switching input device");

        let old = self.audio_handler.clone();
        let mut adjusted_input = input_config;
        adjusted_input.buffer_size = BufferSize::Fixed(self.preferred_buffer_frames);

        let new_handler = AudioHandler::new(
            input,
            old.output_device().clone(),
            adjusted_input,
            old.output_config().clone(),
        );

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
    pub fn set_output_device(&mut self, output: Device, output_config: StreamConfig) {
        info!("Switching output device");

        let old = self.audio_handler.clone();
        let mut adjusted_output = output_config;
        adjusted_output.buffer_size = BufferSize::Fixed(self.preferred_buffer_frames);

        let new_handler = AudioHandler::new(
            old.input_device().clone(),
            output,
            old.input_config().clone(),
            adjusted_output,
        );

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
        if is_on {
            self.start_loopback();
        } else {
            self.stop_loopback();
        }
    }


}
