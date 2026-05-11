use crate::domain::dto::spectrum_snapshot_dto::SpectrumSnapshotDto;
use crate::services::analyzers::spectrum_tap::SpectrumTap;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

/// Lower bound for analyzer frequencies in Hz.
const MIN_ANALYZER_FREQ_HZ: f32 = 20.0;
/// Lower clamp for displayed magnitudes (dBFS).
const MIN_DB: f32 = -90.0;
/// Upper clamp for displayed magnitudes (dBFS).
const MAX_DB: f32 = 6.0;
/// Number of points emitted to the frontend per frame.
const ANALYZER_BINS: usize = 96;

/// Stateless service that converts time-domain tap samples into log-spaced dB spectrum data.
pub struct SpectrumAnalyzerService;

impl SpectrumAnalyzerService {
    /// Builds a spectrum snapshot from the most recent samples in the tap.
    pub fn analyze_tap(tap: &SpectrumTap) -> SpectrumSnapshotDto {
        let sample_rate_hz = tap.sample_rate_hz();
        let samples = tap.snapshot_window();
        Self::analyze_samples(&samples, sample_rate_hz)
    }

    /// Computes FFT magnitudes at log-spaced frequencies and returns a frontend DTO.
    fn analyze_samples(samples: &[f32], sample_rate_hz: u32) -> SpectrumSnapshotDto {
        if samples.is_empty() {
            return SpectrumSnapshotDto {
                sample_rate_hz: sample_rate_hz.max(1),
                frequencies_hz: vec![MIN_ANALYZER_FREQ_HZ; ANALYZER_BINS],
                magnitudes: vec![MIN_DB; ANALYZER_BINS],
                level_db: MIN_DB,
            };
        }

        let sample_rate = sample_rate_hz.max(1) as f32;
        let max_frequency_hz = (sample_rate * 0.5)
            .max(MIN_ANALYZER_FREQ_HZ + 1.0)
            .min(20_000.0);

        let mut fft_input: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, sample)| Complex::new(*sample * hann_window(i, samples.len()), 0.0))
            .collect();

        let mut planner = FftPlanner::<f32>::new();
        planner
            .plan_fft_forward(samples.len())
            .process(&mut fft_input);

        let frequencies_hz: Vec<f32> = (0..ANALYZER_BINS)
            .map(|index| {
                frequency_for_bin(index, ANALYZER_BINS, MIN_ANALYZER_FREQ_HZ, max_frequency_hz)
            })
            .collect();

        let magnitudes: Vec<f32> = frequencies_hz
            .iter()
            .map(|frequency_hz| magnitude_db_at_frequency(&fft_input, sample_rate, *frequency_hz))
            .collect();

        SpectrumSnapshotDto {
            sample_rate_hz: sample_rate_hz.max(1),
            frequencies_hz,
            magnitudes,
            level_db: rms_db(samples),
        }
    }
}

/// Reads one FFT bin nearest to the target frequency and returns clamped dBFS.
fn magnitude_db_at_frequency(
    spectrum: &[Complex<f32>],
    sample_rate: f32,
    frequency_hz: f32,
) -> f32 {
    let n = spectrum.len().max(2);
    let half = n / 2;
    let bin_index = ((frequency_hz / sample_rate) * n as f32)
        .round()
        .clamp(1.0, (half - 1) as f32) as usize;

    let normalized = (2.0 * spectrum[bin_index].norm()) / n as f32;
    (20.0 * normalized.max(1e-7).log10()).clamp(MIN_DB, MAX_DB)
}

/// Returns the geometric center frequency for a log-spaced analyzer bin.
fn frequency_for_bin(index: usize, bin_count: usize, min_hz: f32, max_hz: f32) -> f32 {
    let ratio = max_hz / min_hz;
    let center = (index as f32 + 0.5) / bin_count as f32;
    min_hz * ratio.powf(center)
}

/// Hann window coefficient for sample `index` in a window of length `len`.
fn hann_window(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }

    let phase = (2.0 * std::f32::consts::PI * index as f32) / (len as f32 - 1.0);
    0.5 * (1.0 - phase.cos())
}

/// Computes whole-window RMS in dBFS for level metering.
fn rms_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return MIN_DB;
    }

    let mean_square =
        samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32;
    let rms = mean_square.sqrt();
    (20.0 * rms.max(1e-7).log10()).max(MIN_DB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn analyze_samples_returns_expected_bin_shape() {
            let samples = vec![0.0_f32; 2048];
            let snapshot = SpectrumAnalyzerService::analyze_samples(&samples, 48_000);

            assert_eq!(snapshot.sample_rate_hz, 48_000);
            assert_eq!(snapshot.magnitudes.len(), ANALYZER_BINS);
            assert_eq!(snapshot.frequencies_hz.len(), ANALYZER_BINS);
            assert!(snapshot.level_db <= 0.0);
            assert!(snapshot.level_db >= MIN_DB);
        }

        #[test]
        fn analyze_samples_detects_a_tone_peak() {
            let sample_rate = 48_000.0;
            let target_freq = 1_000.0;
            let samples = (0..2048)
                .map(|n| {
                    (2.0 * std::f32::consts::PI * target_freq * (n as f32 / sample_rate)).sin()
                        * 0.8
                })
                .collect::<Vec<_>>();

            let snapshot = SpectrumAnalyzerService::analyze_samples(&samples, sample_rate as u32);
            let peak_value = snapshot.magnitudes.iter().copied().fold(MIN_DB, f32::max);

            assert!(peak_value > -35.0);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn analyze_samples_with_empty_input_returns_safe_defaults() {
            let snapshot = SpectrumAnalyzerService::analyze_samples(&[], 0);

            assert_eq!(snapshot.sample_rate_hz, 1);
            assert_eq!(snapshot.magnitudes.len(), ANALYZER_BINS);
            assert!(snapshot.magnitudes.iter().all(|value| *value == MIN_DB));
            assert_eq!(snapshot.level_db, MIN_DB);
        }
    }
}
