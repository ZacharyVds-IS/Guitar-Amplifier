//! Latency-oriented measurement helpers for [`AudioService`].
//!
//! This module keeps profiling and latency-estimation logic separate from the
//! real-time loopback orchestration in `audio_service.rs`.

use crate::domain::dto::algorithmic_latency_dto::AlgorithmicLatencyDto;
use crate::domain::dto::buffer_latency_dto::BufferLatencyDto;
use crate::domain::dto::execution_timing_dto::ExecutionTimingDto;
use crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto;
use crate::infrastructure::audio_handler::AudioHandlerTrait;
use crate::services::analyzers::LatencyAnalyzer::LatencyAnalyzer;
use crate::services::audio_service::AudioService;
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use crate::services::round_trip_latency_session::RoundTripLatencySession;
use cpal::BufferSize;
use std::time::Duration;

pub struct AudioLatencyMeasurementService;

impl AudioLatencyMeasurementService {
    /// Measures gain processor execution cost in microseconds per sample.
    pub fn measure_gain_latency(audio_service: &AudioService, block_size: usize) -> f64 {
        let mut gain = GainProcessor::new(audio_service.channel().gain());
        LatencyAnalyzer::measure_effect_added_execution_us(&mut gain, 256, block_size)
    }

    /// Measures tone stack processor execution cost in microseconds per sample.
    pub fn measure_tone_stack_latency(audio_service: &AudioService, block_size: usize) -> f64 {
        let mut tone_stack = ToneStackProcessor::new(audio_service.channel().tone_stack());
        LatencyAnalyzer::measure_effect_added_execution_us(&mut tone_stack, 256, block_size)
    }

    /// Measures execution cost of all processors in the loopback DSP chain.
    ///
    /// Returns a vector of timing measurements in the order they appear in the chain:
    /// 1. Gain
    /// 2. Tone Stack
    /// 3. Master Volume
    pub fn measure_all_dsp_timings(audio_service: &AudioService, block_size: usize) -> Vec<ExecutionTimingDto> {
        let gain_us = Self::measure_gain_latency(audio_service, block_size);
        let tone_stack_us = Self::measure_tone_stack_latency(audio_service, block_size);
        let master_volume_us = {
            let mut master_volume = GainProcessor::new(audio_service.channel().master_volume());
            LatencyAnalyzer::measure_effect_added_execution_us(&mut master_volume, 256, block_size)
        };

        vec![
            ExecutionTimingDto::new("Gain", gain_us),
            ExecutionTimingDto::new("Tone Stack", tone_stack_us),
            ExecutionTimingDto::new("Master Volume", master_volume_us),
        ]
    }

    /// Returns algorithmic latency for all processors in the DSP chain.
    ///
    /// Algorithmic latency is delay introduced by effect design (samples/ms),
    /// not CPU execution time. For the current Gain/Tone Stack/Master chain,
    /// this is zero samples because no processor uses lookahead or delay lines.
    pub fn measure_all_dsp_algorithmic_latency(audio_service: &AudioService) -> Vec<AlgorithmicLatencyDto> {
        let sample_rate_hz = audio_service.audio_handler().output_sample_rate();

        vec![
            AlgorithmicLatencyDto::new("Gain", 0, sample_rate_hz),
            AlgorithmicLatencyDto::new("Tone Stack", 0, sample_rate_hz),
            AlgorithmicLatencyDto::new("Master Volume", 0, sample_rate_hz),
        ]
    }

    /// Returns estimated I/O buffer latency from current input/output stream configs.
    ///
    /// If CPAL uses `BufferSize::Default`, this uses a conservative fallback of 256 frames
    /// for the estimate so UI can still display a practical value.
    pub fn measure_buffer_latency(audio_service: &AudioService) -> BufferLatencyDto {
        const DEFAULT_BUFFER_FRAMES_FALLBACK: u32 = 256;

        let input_frames = match audio_service.audio_handler().input_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => DEFAULT_BUFFER_FRAMES_FALLBACK,
        };

        let output_frames = match audio_service.audio_handler().output_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => DEFAULT_BUFFER_FRAMES_FALLBACK,
        };

        let input_ms = (input_frames as f64 / audio_service.audio_handler().input_sample_rate() as f64) * 1000.0;
        let output_ms = (output_frames as f64 / audio_service.audio_handler().output_sample_rate() as f64) * 1000.0;

        BufferLatencyDto::new(input_ms, output_ms)
    }

    /// Measures round-trip latency using a **dedicated** pair of CPAL streams.
    ///
    /// Opens its own input and output streams (independent of the regular loopback),
    /// warms them up, fires impulses, and returns the averaged result.  Because the
    /// `Mutex<AudioService>` is released by the caller before this runs, the rest of
    /// the UI stays responsive during the measurement.
    pub fn measure_round_trip_latency(handler: &dyn AudioHandlerTrait) -> RoundTripLatencyDto {
        match RoundTripLatencySession::run(handler, Duration::from_secs(10), Duration::from_millis(1500)) {
            Ok(latency_ms) => RoundTripLatencyDto::success(latency_ms),
            Err(error) => RoundTripLatencyDto::failure(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::services::audio_service::AudioService;
    use cpal::StreamConfig;
    use std::sync::Arc;

    fn build_service_with_buffer_config(
        input_rate: u32,
        output_rate: u32,
        input_buffer_size: BufferSize,
        output_buffer_size: BufferSize,
    ) -> AudioService {
        let mut mock = MockAudioHandlerTrait::new();

        let input_config = StreamConfig {
            channels: 1,
            sample_rate: input_rate,
            buffer_size: input_buffer_size,
        };

        let output_config = StreamConfig {
            channels: 1,
            sample_rate: output_rate,
            buffer_size: output_buffer_size,
        };

        mock.expect_input_sample_rate().return_const(input_rate);
        mock.expect_output_sample_rate().return_const(output_rate);
        mock.expect_input_config().return_const(input_config);
        mock.expect_output_config().return_const(output_config);

        AudioService::new_with_handler(Arc::new(mock))
    }

    fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "expected {actual} ~= {expected} (epsilon {epsilon})"
        );
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn measure_all_dsp_timings_returns_expected_processors() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Fixed(256),
                BufferSize::Fixed(256),
            );

            let timings = AudioLatencyMeasurementService::measure_all_dsp_timings(&service, 512);

            assert_eq!(timings.len(), 3);
            assert_eq!(timings[0].processor_name, "Gain");
            assert_eq!(timings[1].processor_name, "Tone Stack");
            assert_eq!(timings[2].processor_name, "Master Volume");
            assert!(timings.iter().all(|t| t.execution_us_per_sample.is_finite()));
            assert!(timings.iter().all(|t| t.execution_us_per_sample >= 0.0));
        }

        #[test]
        fn measure_all_dsp_algorithmic_latency_is_zero_for_simple_chain() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Fixed(256),
                BufferSize::Fixed(256),
            );

            let latency = AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&service);

            assert_eq!(latency.len(), 3);
            assert_eq!(latency[0].processor_name, "Gain");
            assert_eq!(latency[1].processor_name, "Tone Stack");
            assert_eq!(latency[2].processor_name, "Master Volume");
            assert!(latency.iter().all(|item| item.latency_samples == 0));
            assert!(latency.iter().all(|item| item.latency_ms == 0.0));
        }

        #[test]
        fn measure_buffer_latency_uses_fixed_buffer_sizes() {
            let service = build_service_with_buffer_config(
                48_000,
                96_000,
                BufferSize::Fixed(480),
                BufferSize::Fixed(960),
            );

            let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);

            assert_approx_eq(latency.input_buffer_latency_ms, 10.0, 1e-9);
            assert_approx_eq(latency.output_buffer_latency_ms, 10.0, 1e-9);
            assert_approx_eq(latency.total_buffer_latency_ms, 20.0, 1e-9);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn measure_buffer_latency_falls_back_for_default_buffer_size() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Default,
                BufferSize::Default,
            );

            let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
            let expected_single_side_ms = (256.0 / 48_000.0) * 1000.0;

            assert_approx_eq(latency.input_buffer_latency_ms, expected_single_side_ms, 1e-9);
            assert_approx_eq(latency.output_buffer_latency_ms, expected_single_side_ms, 1e-9);
            assert_approx_eq(latency.total_buffer_latency_ms, expected_single_side_ms * 2.0, 1e-9);
        }
    }
}

