use crate::domain::dto::algorithmic_latency_dto::AlgorithmicLatencyDto;
use crate::domain::dto::buffer_latency_dto::BufferLatencyDto;
use crate::domain::dto::execution_timing_dto::ExecutionTimingDto;
use crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto;
use crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

/// Measures gain processor execution impact in microseconds per sample.
#[tauri::command]
pub fn test_gain_latency(audio_service: tauri::State<'_, Mutex<AudioService>>) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let added_us_per_sample = AudioLatencyMeasurementService::measure_gain_latency(&service, 2048);

    info!(
        "Gain processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );
    println!(
        "Gain processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );

    Ok(())
}

/// Measures execution impact of all processors in the DSP chain.
///
/// Returns a vector of CPU-time measurements in chain order:
/// 1. Gain
/// 2. Tone Stack
/// 3. Master Volume
#[tauri::command]
pub fn measure_all_dsp_cpu_timings(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<Vec<ExecutionTimingDto>, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let timings = AudioLatencyMeasurementService::measure_all_dsp_timings(&service, 2048);

    for timing in &timings {
        info!(
            processor = timing.processor_name,
            execution_us_per_sample = timing.execution_us_per_sample,
            "DSP chain processor timing"
        );
    }

    Ok(timings)
}

#[tauri::command]
pub fn measure_all_dsp_algorithmic_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<Vec<AlgorithmicLatencyDto>, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let latency = AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&service);

    for item in &latency {
        info!(
            processor = item.processor_name,
            latency_samples = item.latency_samples,
            latency_ms = item.latency_ms,
            "DSP chain processor algorithmic latency"
        );
    }

    Ok(latency)
}
#[tauri::command]
pub fn measure_buffer_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<BufferLatencyDto, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);

    info!(
        input_buffer_latency_ms = latency.input_buffer_latency_ms,
        output_buffer_latency_ms = latency.output_buffer_latency_ms,
        total_buffer_latency_ms = latency.total_buffer_latency_ms,
        "I/O buffer latency"
    );

    Ok(latency)
}

#[tauri::command]
pub fn measure_round_trip_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<RoundTripLatencyDto, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let latency = AudioLatencyMeasurementService::measure_round_trip_latency(&service);

    if latency.is_valid {
        info!(
            round_trip_latency_ms = latency.latency_ms,
            "Round-trip latency measurement"
        );
    } else {
        info!(
            error = latency.error.clone(),
            "Round-trip latency measurement failed"
        );
    }

    Ok(latency)
}
