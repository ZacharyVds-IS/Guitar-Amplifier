use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tracing::{error, info, warn};

const MAX_RESAMPLE_RATIO_RELATIVE: f64 = 2.0;
const SINC_LEN: usize = 128;
const OVERSAMPLING_FACTOR: usize = 128;
const CUTOFF: f32 = 0.95;

pub struct RubatoResampler {
    inner: SincFixedIn<f32>,
    input_chunk_size: usize,
    input_buffer: Vec<f32>,
}

impl RubatoResampler {
    pub fn new(input_rate: u32, output_rate: u32, input_chunk_size: usize) -> Result<Self, String> {
        if input_rate == 0 || output_rate == 0 {
            return Err("Sample rates must be > 0".to_string());
        }

        if input_chunk_size == 0 {
            return Err("Chunk size must be > 0".to_string());
        }

        let params = SincInterpolationParameters {
            sinc_len: SINC_LEN,
            f_cutoff: CUTOFF,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: OVERSAMPLING_FACTOR,
            window: WindowFunction::BlackmanHarris2,
        };

        let ratio = output_rate as f64 / input_rate as f64;
        let inner = SincFixedIn::<f32>::new(
            ratio,
            MAX_RESAMPLE_RATIO_RELATIVE,
            params,
            input_chunk_size,
            1,
        )
        .map_err(|e| format!("Failed to create rubato resampler: {e}"))?;

        Ok(Self {
            inner,
            input_chunk_size,
            input_buffer: Vec::with_capacity(input_chunk_size),
        })
    }

    pub fn process_sample(&mut self, sample: f32) -> Vec<f32> {
        self.input_buffer.push(sample);

        if self.input_buffer.len() < self.input_chunk_size {
            return Vec::new();
        }

        let chunk: Vec<f32> = self.input_buffer.drain(..self.input_chunk_size).collect();
        self.process_chunk(chunk)
    }

    pub fn flush(&mut self) -> Vec<f32> {
        if self.input_buffer.is_empty() {
            return Vec::new();
        }

        let mut padded_chunk: Vec<f32> = self.input_buffer.drain(..).collect();
        padded_chunk.resize(self.input_chunk_size, 0.0);
        self.process_chunk(padded_chunk)
    }

    fn process_chunk(&mut self, input_chunk: Vec<f32>) -> Vec<f32> {
        let input = vec![input_chunk];

        match self.inner.process(&input, None) {
            Ok(output) => output.into_iter().next().unwrap_or_default(),
            Err(e) => {
                error!("Rubato processing failed: {e}");
                Vec::new()
            }
        }
    }
}

/// Determines when resampling occurs relative to the DSP chain based on the
/// input and output sample rates.
///
/// - [`Bypass`]: rates are equal, no resampling overhead at all.
/// - [`PreDsp`]: input rate is higher than output rate — downsample **before**
///   the DSP chain so every gain/EQ calculation runs at the lower output rate.
/// - [`PostDsp`]: input rate is lower than output rate — run DSP at the cheaper
///   input rate and upsample **after**, just before pushing to the output buffer.
///
/// [`Bypass`]: ResamplePolicy::Bypass
/// [`PreDsp`]: ResamplePolicy::PreDsp
/// [`PostDsp`]: ResamplePolicy::PostDsp
pub enum ResamplePolicy {
    Bypass,
    PreDsp(RubatoResampler),
    PostDsp(RubatoResampler),
}

impl ResamplePolicy {
    /// Selects and initialises the correct resampling policy for the given rates.
    ///
    /// Falls back to [`Bypass`] with a warning if the resampler fails to initialise.
    ///
    /// [`Bypass`]: ResamplePolicy::Bypass
    pub fn from_rates(input_rate: u32, output_rate: u32, chunk_size: usize) -> Self {
        match input_rate.cmp(&output_rate) {
            std::cmp::Ordering::Equal => {
                info!("Sample rate is equal ({input_rate} Hz) — no resampling needed");
                Self::Bypass
            }
            std::cmp::Ordering::Greater => {
                info!(
                    "Sample rates differ: input ({input_rate} Hz) > output ({output_rate} Hz) — downsampling before DSP"
                );
                match RubatoResampler::new(input_rate, output_rate, chunk_size) {
                    Ok(r) => Self::PreDsp(r),
                    Err(e) => {
                        warn!("Failed to initialise pre-DSP downsampler, using bypass: {e}");
                        Self::Bypass
                    }
                }
            }
            std::cmp::Ordering::Less => {
                info!(
                    "Sample rates differ: input ({input_rate} Hz) < output ({output_rate} Hz) — upsampling after DSP"
                );
                match RubatoResampler::new(input_rate, output_rate, chunk_size) {
                    Ok(r) => Self::PostDsp(r),
                    Err(e) => {
                        warn!("Failed to initialise post-DSP upsampler, using bypass: {e}");
                        Self::Bypass
                    }
                }
            }
        }
    }

    /// Processes a single input sample through the resampling policy.
    ///
    /// `dsp` is called once per sample that enters the DSP chain.
    /// Returns zero or more output samples ready to be pushed to the output buffer.
    pub fn process(&mut self, sample: f32, dsp: &mut impl FnMut(f32) -> f32) -> Vec<f32> {
        match self {
            Self::Bypass => vec![dsp(sample)],
            Self::PreDsp(resampler) => resampler
                .process_sample(sample)
                .into_iter()
                .map(|s| dsp(s))
                .collect(),
            Self::PostDsp(resampler) => resampler.process_sample(dsp(sample)),
        }
    }

    /// Flushes any remaining buffered samples through the policy at shutdown.
    pub fn flush(&mut self, dsp: &mut impl FnMut(f32) -> f32) -> Vec<f32> {
        match self {
            Self::Bypass => Vec::new(),
            Self::PreDsp(resampler) => resampler
                .flush()
                .into_iter()
                .map(|s| dsp(s))
                .collect(),
            Self::PostDsp(resampler) => resampler.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rubato_resampler_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn upsampling_eventually_produces_output_samples() {
                let mut resampler = RubatoResampler::new(44_100, 48_000, 32).unwrap();
                let mut produced = 0usize;

                for _ in 0..512 {
                    produced += resampler.process_sample(0.2).len();
                }
                produced += resampler.flush().len();

                assert!(produced > 0, "Upsampler should eventually produce output samples");
            }

            #[test]
            fn downsampling_outputs_fewer_samples_than_input() {
                let mut resampler = RubatoResampler::new(48_000, 44_100, 32).unwrap();
                let mut produced = 0usize;

                for _ in 0..128 {
                    produced += resampler.process_sample(0.2).len();
                }
                produced += resampler.flush().len();

                assert!(produced < 128, "Downsampler should produce fewer samples than consumed");
            }

            #[test]
            fn process_sample_buffers_until_chunk_is_full() {
                let chunk_size = 16;
                let mut resampler = RubatoResampler::new(44_100, 48_000, chunk_size).unwrap();
                for _ in 0..(chunk_size - 1) {
                    assert!(resampler.process_sample(0.1).is_empty(), "Should buffer until chunk is full");
                }

                let _ = resampler.process_sample(0.1);

                let mut eventually_produced = false;
                for _ in 0..(chunk_size * 8) {
                    if !resampler.process_sample(0.1).is_empty() {
                        eventually_produced = true;
                        break;
                    }
                }

                assert!(
                    eventually_produced,
                    "Resampler should eventually produce output after receiving enough samples"
                );
            }

            #[test]
            fn flush_clears_remaining_buffered_input() {
                let chunk_size = 32;
                let mut resampler = RubatoResampler::new(44_100, 48_000, chunk_size).unwrap();
                for _ in 0..10 {
                    resampler.process_sample(0.1);
                }

                assert!(!resampler.input_buffer.is_empty(), "Input buffer should contain pending samples before flush");
                let _ = resampler.flush();
                assert!(resampler.input_buffer.is_empty(), "Flush should clear pending input samples");
            }

            #[test]
            fn flush_on_empty_buffer_returns_nothing() {
                let mut resampler = RubatoResampler::new(44_100, 48_000, 32).unwrap();
                let flushed = resampler.flush();
                assert!(flushed.is_empty(), "Flush on an empty buffer should return no samples");
            }
        }

        mod failure_path {
            use super::*;

            #[test]
            fn zero_input_rate_returns_error() {
                let result = RubatoResampler::new(0, 48_000, 32);
                assert!(result.is_err(), "Zero input rate should return an error");
            }

            #[test]
            fn zero_output_rate_returns_error() {
                let result = RubatoResampler::new(44_100, 0, 32);
                assert!(result.is_err(), "Zero output rate should return an error");
            }

            #[test]
            fn zero_chunk_size_returns_error() {
                let result = RubatoResampler::new(44_100, 48_000, 0);
                assert!(result.is_err(), "Zero chunk size should return an error");
            }
        }
    }

    mod resample_policy_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn equal_rates_selects_bypass() {
                let policy = ResamplePolicy::from_rates(48_000, 48_000, 32);
                assert!(matches!(policy, ResamplePolicy::Bypass));
            }

            #[test]
            fn higher_input_rate_selects_pre_dsp() {
                let policy = ResamplePolicy::from_rates(48_000, 44_100, 32);
                assert!(matches!(policy, ResamplePolicy::PreDsp(_)));
            }

            #[test]
            fn lower_input_rate_selects_post_dsp() {
                let policy = ResamplePolicy::from_rates(44_100, 48_000, 32);
                assert!(matches!(policy, ResamplePolicy::PostDsp(_)));
            }

            #[test]
            fn bypass_process_applies_dsp_and_returns_one_sample() {
                let mut policy = ResamplePolicy::Bypass;
                let result = policy.process(0.5, &mut |s| s * 2.0);

                assert_eq!(result.len(), 1);
                assert!((result[0] - 1.0).abs() < 1e-6, "Bypass should apply DSP directly");
            }

            #[test]
            fn bypass_flush_returns_empty() {
                let mut policy = ResamplePolicy::Bypass;
                let result = policy.flush(&mut |s| s);

                assert!(result.is_empty(), "Bypass flush should always return nothing");
            }

            #[test]
            fn pre_dsp_applies_dsp_to_resampled_output() {
                let mut policy = ResamplePolicy::from_rates(48_000, 44_100, 32);
                let mut dsp_called = false;

                for _ in 0..1024 {
                    policy.process(0.5, &mut |s| {
                        dsp_called = true;
                        s
                    });
                }

                let _ = policy.flush(&mut |s| {
                    dsp_called = true;
                    s
                });

                assert!(dsp_called, "PreDsp should call DSP on the downsampled samples");
            }

            #[test]
            fn post_dsp_applies_dsp_before_resampling() {
                let mut policy = ResamplePolicy::from_rates(44_100, 48_000, 32);
                let mut dsp_call_count = 0;
                let input_count = 128;

                for _ in 0..input_count {
                    policy.process(0.5, &mut |s| {
                        dsp_call_count += 1;
                        s
                    });
                }
                assert_eq!(dsp_call_count, input_count, "PostDsp should call DSP once per input sample");
            }
        }

        mod failure_path {
            use super::*;

            #[test]
            fn invalid_rates_fall_back_to_bypass() {
                let policy = ResamplePolicy::from_rates(0, 48_000, 32);
                assert!(matches!(policy, ResamplePolicy::Bypass), "Invalid rates should fall back to Bypass");
            }

            #[test]
            fn zero_chunk_size_falls_back_to_bypass() {
                let policy = ResamplePolicy::from_rates(44_100, 48_000, 0);
                assert!(matches!(policy, ResamplePolicy::Bypass), "Zero chunk size should fall back to Bypass");
            }
        }
    }
}