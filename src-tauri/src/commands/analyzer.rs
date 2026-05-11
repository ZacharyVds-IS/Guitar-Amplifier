use crate::domain::dto::spectrum_snapshot_dto::SpectrumSnapshotDto;
use crate::services::analyzers::spectrum_analyzer_service::SpectrumAnalyzerService;
use crate::services::audio_service::AudioService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::time::{interval, Duration};

/// Tauri event name emitted by the backend when a new spectrum frame is available.
const LIVE_SPECTRUM_EVENT: &str = "live-spectrum";
/// Target interval for push-streamed spectrum frames (about 60 FPS).
const STREAM_INTERVAL_MS: u64 = 16;

/// Shared state for the analyzer stream task.
///
/// The task is started by `start_live_spectrum_stream` and stopped by either
/// `stop_live_spectrum_stream` or when the target window can no longer receive events.
///
/// An `AtomicBool` shutdown flag is used so the async task can exit cleanly on the
/// next tick without relying on `JoinHandle::abort`, which cannot forcibly stop a
/// blocking thread spawned by `spawn_blocking`.
#[derive(Default)]
pub struct SpectrumStreamState {
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    shutdown: Arc<AtomicBool>,
}

/// Returns a single, immediate spectrum snapshot.
///
/// This command is useful for first paint / fallback reads before the push stream
/// starts delivering `live-spectrum` events.
///
/// FFT analysis is offloaded to a blocking task so the async command handler never
/// stalls the Tauri runtime thread.
#[tauri::command]
pub async fn get_live_spectrum(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<SpectrumSnapshotDto, String> {
    let tap = {
        let service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        service.spectrum_tap().clone()
    };

    tauri::async_runtime::spawn_blocking(move || SpectrumAnalyzerService::analyze_tap(tap.as_ref()))
        .await
        .map_err(|e| format!("FFT analysis task failed: {e}"))
}

/// Starts (or restarts) push-based live spectrum streaming for the calling window.
///
/// Behavior:
/// - Captures the current shared `SpectrumTap` from `AudioService`.
/// - Signals any previously running stream task to shut down via an `AtomicBool` flag,
///   then replaces it. This avoids leaking threads that `JoinHandle::abort` cannot stop.
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

    // Signal the previous task to stop on its next tick before replacing it.
    stream_state.shutdown.store(true, Ordering::Relaxed);
    stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .take();

    let shutdown = Arc::clone(&stream_state.shutdown);
    shutdown.store(false, Ordering::Relaxed);

    let handle = tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_millis(STREAM_INTERVAL_MS));
        loop {
            ticker.tick().await;
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            let tap_ref = Arc::clone(&tap);
            let snapshot = tauri::async_runtime::spawn_blocking(move || {
                SpectrumAnalyzerService::analyze_tap(tap_ref.as_ref())
            })
            .await;
            match snapshot {
                Ok(data) => {
                    if window.emit(LIVE_SPECTRUM_EVENT, data).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
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
/// Sets the shutdown flag so the async loop exits cleanly on its next tick.
#[tauri::command]
pub fn stop_live_spectrum_stream(
    stream_state: tauri::State<'_, SpectrumStreamState>,
) -> Result<(), String> {
    stream_state.shutdown.store(true, Ordering::Relaxed);
    stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .take();

    Ok(())
}
