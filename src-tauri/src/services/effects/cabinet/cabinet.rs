use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::infrastructure::file_loader::{FileLoader, FileLoaderTrait};
use crate::services::processors::resampler::resampler::ResamplerImpl;
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

/// Default cabinet impulse-response WAV file loaded when no explicit profile is supplied.
const DEFAULT_IR_FILE: &str = "info-support-halway.wav";
/// Chunk size used by IR resampling during initialization.
const IR_RESAMPLER_CHUNK_SIZE: usize = 256;
/// Number of input samples collected before one FFT convolution pass.
const CONV_BLOCK_SIZE: usize = 256;
/// Upper bound for IR length to keep real-time CPU usage predictable.
const MAX_IR_SAMPLES: usize = 2048;
/// Safety clamp applied to processed output to reduce hard digital clipping.
const OUTPUT_CLAMP: f32 = 0.98;

/// FFT-based cabinet simulator that convolves input audio with a loaded IR.
///
/// The effect uses block convolution:
/// - gather `CONV_BLOCK_SIZE` input samples,
/// - forward FFT,
/// - multiply by precomputed IR FFT kernel,
/// - inverse FFT,
/// - overlap-add into an output queue.
///
/// `overlap-add` means each processed block contributes some samples that belong
/// to the same time positions as the next block. Instead of overwriting those
/// positions, we add them together in `output_queue`.
/// This reconstructs the same linear-convolution result you would get from a
/// full sample-by-sample convolution, but in a block-friendly way.
///
/// Public-facing behavior is sample-by-sample through [`AudioProcessor::process`],
/// while internally processing is block-based for efficiency.
pub struct Cabinet {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    color: String,
    ir_file_path: String,
    ir_buffer: Vec<f32>,
    ir_fft_kernel: Vec<Complex<f32>>,
    ir_fft_size: usize,
    fft_forward: Arc<dyn Fft<f32>>,
    fft_inverse: Arc<dyn Fft<f32>>,
    fft_scratch: Vec<Complex<f32>>,
    cabinet_gain: f32,
    has_logged_ir_unavailable: bool,
    input_block: Vec<f32>,
    output_queue: VecDeque<f32>,
    dsp_sample_rate: u32,
}

impl Cabinet {
    /// Creates a new cabinet effect instance and prepares FFT convolution state.
    ///
    /// Initialization steps:
    /// - load default IR file,
    /// - optionally resample IR to the DSP sample rate,
    /// - truncate very long IRs,
    /// - precompute IR FFT kernel,
    /// - preallocate buffers used by the audio thread.
    pub fn new(
        id: u32,
        name: String,
        is_active: bool,
        color: String,
        ir_file_path: String,
        dsp_sample_rate: u32,
    ) -> Self {
        info!("init cabinet simulation");
        let file_loader = FileLoader::new();

        let selected_ir_file = if ir_file_path.trim().is_empty() {
            DEFAULT_IR_FILE.to_string()
        } else {
            ir_file_path
        };

        let temp_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("default_ir")
            .join(&selected_ir_file);

        let ir_buffer = file_loader.read_wav_to_buffer(&temp_file_path);
        let ir_sample_rate = file_loader
            .read_wav_sample_rate(&temp_file_path)
            .unwrap_or(dsp_sample_rate);
        let (mut ir_buffer, resampling_applied) =
            Self::resample_if_needed(ir_buffer, ir_sample_rate, dsp_sample_rate);
        if ir_buffer.is_empty() {
            warn!(
                "Cabinet IR buffer is empty. IR file may be missing, unreadable, unsupported, or corrupt. Falling back to passthrough."
            );
        }
        if ir_buffer.len() > MAX_IR_SAMPLES {
            info!(
                "Cabinet IR too long ({} samples). Truncating to {} to keep real-time CPU stable.",
                ir_buffer.len(),
                MAX_IR_SAMPLES
            );
            ir_buffer.truncate(MAX_IR_SAMPLES);
        }
        let ir_fft_size = (CONV_BLOCK_SIZE + ir_buffer.len().saturating_sub(1))
            .next_power_of_two()
            .max(2);
        let output_queue_capacity = CONV_BLOCK_SIZE + ir_buffer.len();
        let (fft_forward, fft_inverse) = Self::build_fft_plans(ir_fft_size);
        let ir_fft_kernel = Self::convert_ir_to_fft_kernel(&ir_buffer, ir_fft_size, &fft_forward);
        let cabinet_gain = Self::compute_cabinet_gain(&ir_buffer);

        info!(
      "Cabinet rates -> ir_sample_rate={}, dsp_sample_rate={}, resampling_applied={}, ir_len={}, fft_size={}, cabinet_gain={}",
			ir_sample_rate,
			dsp_sample_rate,
			resampling_applied,
      ir_buffer.len(),
      ir_fft_size,
      cabinet_gain
		);

        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            color,
            ir_file_path: selected_ir_file,
            ir_buffer,
            ir_fft_kernel,
            ir_fft_size,
            fft_forward,
            fft_inverse,
            fft_scratch: vec![Complex::new(0.0_f32, 0.0_f32); ir_fft_size],
            cabinet_gain,
            has_logged_ir_unavailable: false,
            input_block: Vec::with_capacity(CONV_BLOCK_SIZE),
            output_queue: VecDeque::with_capacity(output_queue_capacity),
            dsp_sample_rate,
        }
    }

    /// Computes a conservative gain factor from IR peak amplitude.
    ///
    /// If IR peak is above unity, this returns an attenuation factor `1.0 / peak`.
    /// Otherwise returns `1.0`.
    fn compute_cabinet_gain(ir_buffer: &[f32]) -> f32 {
        let peak = ir_buffer
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        if peak > 1.0 {
            1.0 / peak
        } else {
            1.0
        }
    }

    /// Builds forward and inverse FFT plans for a fixed FFT size.
    fn build_fft_plans(fft_size: usize) -> (Arc<dyn Fft<f32>>, Arc<dyn Fft<f32>>) {
        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(fft_size);
        let inverse = planner.plan_fft_inverse(fft_size);
        (forward, inverse)
    }

    /// Converts the time-domain IR into a frequency-domain convolution kernel.
    ///
    /// The returned vector has length `fft_size` and is zero-padded when IR is shorter.
    fn convert_ir_to_fft_kernel(
        ir_buffer: &[f32],
        fft_size: usize,
        fft_forward: &Arc<dyn Fft<f32>>,
    ) -> Vec<Complex<f32>> {
        if ir_buffer.is_empty() {
            return Vec::new();
        }

        let mut buffer = vec![Complex::new(0.0_f32, 0.0_f32); fft_size];
        for (idx, sample) in ir_buffer.iter().enumerate() {
            buffer[idx].re = *sample;
        }

        fft_forward.process(&mut buffer);
        buffer
    }

    /// Pushes dry samples when IR data is unavailable (missing/unreadable/corrupt).
    fn push_passthrough_block_for_ir_unavailable(&mut self) {
        for &sample in &self.input_block {
            self.output_queue.push_back(sample);
        }
    }

    /// Copies the current input block into the reusable FFT scratch buffer.
    ///
    /// The remaining tail is zero-filled to represent block convolution padding.
    fn prepare_fft_input_from_block(&mut self) {
        self.fft_scratch.fill(Complex::new(0.0_f32, 0.0_f32));
        for (sample_index, sample) in self.input_block.iter().enumerate() {
            self.fft_scratch[sample_index].re = *sample;
        }
    }

    /// Applies point-wise complex multiplication `X[k] *= H[k]` in frequency domain.
    fn multiply_input_by_ir_in_frequency_domain(&mut self) {
        for (input_bin, ir_bin) in self.fft_scratch.iter_mut().zip(self.ir_fft_kernel.iter()) {
            *input_bin *= *ir_bin;
        }
    }

    /// Performs overlap-add accumulation of the current IFFT block into output queue.
    ///
    /// This method:
    /// - normalizes by FFT size,
    /// - applies cabinet gain,
    /// - accumulates into queued samples so block boundaries remain continuous.
    ///
    /// Why "add" and not "replace"?
    ///
    /// Convolving one input block with an IR produces an output that is longer than
    /// the input block (`input_len + ir_len - 1`). The tail of the current block
    /// lands in the same timeline region as the start of future blocks.
    ///
    /// If we replaced samples, that tail energy would be lost and you would hear
    /// discontinuities (clicks/crackle) at block edges. By adding into existing
    /// queued values, block outputs stitch together into one continuous signal.
    fn overlap_add_ifft_block_into_queue(&mut self) {
        let fft_normalization = self.ir_fft_size as f32;
        let linear_conv_len = self.input_block.len() + self.ir_buffer.len().saturating_sub(1);

        if self.output_queue.len() < linear_conv_len {
            self.output_queue.resize(linear_conv_len, 0.0);
        }

        for sample_index in 0..linear_conv_len {
            if let Some(output_slot) = self.output_queue.get_mut(sample_index) {
                *output_slot +=
                    (self.fft_scratch[sample_index].re / fft_normalization) * self.cabinet_gain;
            }
        }
    }

    /// Runs one full block convolution pass for the currently buffered input block.
    ///
    /// Signal flow in plain language:
    ///
    /// 1. Put the current time-domain input block (`x`) into the FFT buffer.
    /// 2. Convert it to frequency domain with FFT (`X = FFT(x)`).
    /// 3. Apply cabinet tone by multiplying each frequency bin with the precomputed
    ///    IR kernel (`Y[k] = X[k] * H[k]`).
    /// 4. Convert back to time domain (`y = IFFT(Y)`).
    /// 5. Overlap-add `y` into `output_queue` so neighboring blocks combine correctly.
    ///
    /// Compact form: `x -> FFT(x) -> FFT(x) * FFT(h) -> IFFT -> overlap-add`.
    fn convolve_current_block(&mut self) {
        if self.input_block.is_empty() {
            return;
        }

        if self.ir_fft_kernel.is_empty() {
            if !self.has_logged_ir_unavailable {
                warn!(
                    "Cabinet IR kernel is empty. Using passthrough until a valid IR can be loaded."
                );
                self.has_logged_ir_unavailable = true;
            }
            self.push_passthrough_block_for_ir_unavailable();
            self.input_block.clear();
            return;
        }

        self.prepare_fft_input_from_block();
        self.fft_forward.process(&mut self.fft_scratch);
        self.multiply_input_by_ir_in_frequency_domain();
        self.fft_inverse.process(&mut self.fft_scratch);
        self.overlap_add_ifft_block_into_queue();

        self.input_block.clear();
    }

    /// Resamples an IR buffer when source and target sample rates differ.
    ///
    /// Returns `(buffer, was_resampled)`.
    /// On any resampler setup/processing failure, the original buffer is returned.
    fn resample_if_needed(
        buffer: Vec<f32>,
        source_rate: u32,
        target_rate: u32,
    ) -> (Vec<f32>, bool) {
        if buffer.len() < 2 || source_rate == 0 || target_rate == 0 || source_rate == target_rate {
            return (buffer, false);
        }

        let mut resampler =
            match ResamplerImpl::new(source_rate, target_rate, IR_RESAMPLER_CHUNK_SIZE) {
                Ok(resampler) => resampler,
                Err(err) => {
                    warn!(
					"Failed to initialize cabinet IR resampler ({} -> {}): {}. Using original IR buffer.",
					source_rate,
					target_rate,
					err
				);
                    return (buffer, false);
                }
            };

        let mut out = Vec::new();
        for &sample in &buffer {
            out.extend(resampler.process_sample(sample));
        }
        out.extend(resampler.flush());

        if out.is_empty() {
            warn!(
                "Cabinet IR resampling produced no output ({} -> {}). Using original IR buffer.",
                source_rate, target_rate
            );
            return (buffer, false);
        }

        (out, true)
    }

    /// Returns the sample rate at which cabinet DSP processing runs.
    pub fn sample_rate(&self) -> u32 {
        self.dsp_sample_rate
    }

    /// Returns the precomputed frequency-domain IR kernel.
    ///
    /// Mainly intended for diagnostics and tests.
    pub fn ir_fft_kernel(&self) -> &[Complex<f32>] {
        &self.ir_fft_kernel
    }

    /// Returns the FFT size used for cabinet convolution.
    pub fn ir_fft_size(&self) -> usize {
        self.ir_fft_size
    }
}

impl AudioProcessor for Cabinet {
    /// Processes one sample through the cabinet effect.
    ///
    /// Internally this is block-based:
    /// - one sample is dequeued from `output_queue`,
    /// - one sample is appended to `input_block`,
    /// - when the block is full, a new convolution block is computed.
    ///
    /// If queue underruns, silence is returned until the next block result is available.
    fn process(&mut self, sample: f32) -> f32 {
        let output_sample = self.output_queue.pop_front().unwrap_or(0.0);

        self.input_block.push(sample);
        if self.input_block.len() == CONV_BLOCK_SIZE {
            self.convolve_current_block();
        }

        output_sample.clamp(-OUTPUT_CLAMP, OUTPUT_CLAMP)
    }
}

impl Effect for Cabinet {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn get_color(&self) -> String {
        self.color.clone()
    }

    fn to_dto(&self) -> EffectDto {
        EffectDto::Cabinet(CabinetDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            ir_file_path: self.ir_file_path.clone(),
        })
    }

    fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_active)
    }
}
