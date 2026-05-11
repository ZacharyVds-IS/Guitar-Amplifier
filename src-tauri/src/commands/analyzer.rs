use crate::domain::dto::spectrum_snapshot_dto::SpectrumSnapshotDto;
use crate::services::analyzers::spectrum_analyzer_service::SpectrumAnalyzerService;
use crate::services::audio_service::AudioService;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;

/// Tauri event name emitted by the backend when a new spectrum frame is available.
const LIVE_SPECTRUM_EVENT: &str = "live-spectrum";
/// Target interval for push-streamed spectrum frames (about 60 FPS).
const STREAM_INTERVAL_MS: u64 = 16;

/// Shared state for the analyzer stream task.
///
/// The task is started by `start_live_spectrum_stream` and stopped by either
/// `stop_live_spectrum_stream` or when the target window can no longer receive events.
#[derive(Default)]
pub struct SpectrumStreamState {
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

/// Returns a single, immediate spectrum snapshot.
///
/// This command is useful for first paint / fallback reads before the push stream
/// starts delivering `live-spectrum` events.
#[tauri::command]
pub fn get_live_spectrum(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<SpectrumSnapshotDto, String> {
    let tap = {
        let service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        service.spectrum_tap().clone()
    };

    Ok(SpectrumAnalyzerService::analyze_tap(tap.as_ref()))
}

/// Starts (or restarts) push-based live spectrum streaming for the calling window.
///
/// Behavior:
/// - Captures the current shared `SpectrumTap` from `AudioService`.
/// - Aborts any previously running stream task.
/// - Spawns a background loop that analyzes the tap and emits `live-spectrum`
///   events at `STREAM_INTERVAL_MS` cadence.
/// - Automatically exits when event emission fails (for example, when window closes).
#[tauri::command]
pub fn start_live_spectrum_stream(
    window: tauri::Window,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    stream_state: tauri::State<'_, SpectrumStreamState>,
) -> Result<(), String> {
    let tap: Arc<_> = {
        let service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        service.spectrum_tap().clone()
    };

    if let Some(existing) = stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .take()
    {
        existing.abort();
    }

    let handle = tauri::async_runtime::spawn_blocking(move || loop {
        let snapshot = SpectrumAnalyzerService::analyze_tap(tap.as_ref());
        if window.emit(LIVE_SPECTRUM_EVENT, snapshot).is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(STREAM_INTERVAL_MS));
    });

    stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .replace(handle);

    Ok(())
}

/// Stops the active live spectrum stream task, if one exists.
///
/// This is safe to call repeatedly; when no task is active it becomes a no-op.
#[tauri::command]
pub fn stop_live_spectrum_stream(
    stream_state: tauri::State<'_, SpectrumStreamState>,
) -> Result<(), String> {
    if let Some(existing) = stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .take()
    {
        existing.abort();
    }

    Ok(())
}
