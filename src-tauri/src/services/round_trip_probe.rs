/// Round-trip latency probe.
///
/// **Calibration** — listens to [`CALIBRATION_SAMPLES`] input samples to measure the
/// ambient noise peak, then derives a detection threshold that is always reachable by
/// the outgoing impulse regardless of input volume.
///
/// **Measurement** — fires [`IMPULSE_COUNT`] impulses in sequence, waits for each echo,
/// and reports the averaged round-trip time in milliseconds.
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};


// The only three numbers worth tuning:
const CALIBRATION_SAMPLES: usize = 5;    // ambient samples before firing impulses
const IMPULSE_COUNT: usize = 3;          // impulses per measurement (result is averaged)
const GUARD_SAMPLES: usize = 256;        // dead-time after each impulse (~5.8 ms @ 44 100 Hz)

/// Amplitude of each impulse pushed to the output stream.
pub const IMPULSE_AMPLITUDE: f32 = 0.95;

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbePhase {
    Idle,
    /// Collecting ambient samples to compute the detection threshold.
    CalibrationAmbient,
    /// Waiting for the echo of impulse `i` (0-based).
    WaitingForEcho(usize),
}

#[derive(Default)]
struct ProbeState {
    generation: u64,
    request_pending: bool,
    completed_generation: u64,
    result_ms: Option<f64>,
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// ProbeWorkerContext — owned exclusively by the worker thread, no locking
// ---------------------------------------------------------------------------

pub struct ProbeWorkerContext {
    pub phase: ProbePhase,
    pub generation: u64,
    pub threshold: f32,
    pub impulse_sent_at: Option<Instant>,

    ambient_peak: f32,
    ambient_count: usize,
    guard_remaining: usize,
    echo_durations_ms: Vec<f64>,
}

impl ProbeWorkerContext {
    pub fn new() -> Self {
        Self {
            phase: ProbePhase::Idle,
            generation: 0,
            threshold: 0.0,
            impulse_sent_at: None,
            ambient_peak: 0.0,
            ambient_count: 0,
            guard_remaining: 0,
            echo_durations_ms: Vec::with_capacity(IMPULSE_COUNT),
        }
    }

    /// Start a fresh measurement cycle.
    pub fn begin(&mut self, generation: u64) {
        *self = Self::new();
        self.generation = generation;
        self.phase = ProbePhase::CalibrationAmbient;
        println!("[RT-PROBE] Gen {} — measuring ambient noise ({} samples)…", generation, CALIBRATION_SAMPLES);
    }

    /// Feed one ambient sample. Returns `true` when the threshold has been computed
    /// and the probe is ready for the first impulse.
    pub fn feed_ambient_sample(&mut self, sample: f32) -> bool {
        self.ambient_peak = self.ambient_peak.max(sample.abs());
        self.ambient_count += 1;

        if self.ambient_count < CALIBRATION_SAMPLES {
            return false;
        }

        // Threshold = peak × 2 (above noise), floored at 0.05 (silence guard),
        // capped at half the impulse amplitude so the echo is always detectable.
        self.threshold = (self.ambient_peak * 2.0)
            .max(0.05)
            .min(IMPULSE_AMPLITUDE * 0.5);

        println!(
            "[RT-PROBE] Calibration done. Peak: {:.4}, threshold: {:.4}",
            self.ambient_peak, self.threshold
        );
        true
    }

    /// Call immediately after pushing an impulse to the output buffer.
    /// Starts the clock and arms the dead-time guard.
    pub fn arm_impulse(&mut self) {
        self.impulse_sent_at = Some(Instant::now());
        self.guard_remaining = GUARD_SAMPLES;
    }

    /// Returns `true` when `sample` exceeds the threshold and the guard period has passed.
    pub fn check_echo(&mut self, sample: f32) -> bool {
        if self.guard_remaining > 0 {
            self.guard_remaining -= 1;
            return false;
        }
        sample.abs() >= self.threshold
    }

    /// Record an echo. Returns `Some(avg_ms)` once all impulses have been answered,
    /// `None` if more impulses remain.
    pub fn record_echo(&mut self) -> Option<f64> {
        let sent_at = self.impulse_sent_at.take()?;
        let ms = sent_at.elapsed().as_secs_f64() * 1000.0;
        let idx = match self.phase { ProbePhase::WaitingForEcho(i) => i, _ => return None };

        self.echo_durations_ms.push(ms);
        println!("[RT-PROBE] Echo {}/{}: {:.2} ms", idx + 1, IMPULSE_COUNT, ms);

        if self.echo_durations_ms.len() >= IMPULSE_COUNT {
            let avg = self.echo_durations_ms.iter().sum::<f64>() / self.echo_durations_ms.len() as f64;
            println!("[RT-PROBE] Done. Avg round-trip: {:.2} ms", avg);
            self.phase = ProbePhase::Idle;
            Some(avg)
        } else {
            self.phase = ProbePhase::WaitingForEcho(idx + 1);
            None
        }
    }

    /// `true` if the in-flight impulse has not echoed within 3 seconds.
    pub fn is_timed_out(&self) -> bool {
        self.impulse_sent_at
            .map(|t| t.elapsed() >= Duration::from_secs(3))
            .unwrap_or(false)
    }

    pub fn abort(&mut self) {
        *self = Self::new();
    }

    /// Returns `true` when no measurement is in progress and the probe can accept a new request.
    pub fn is_idle(&self) -> bool {
        self.phase == ProbePhase::Idle
    }

    /// Drive the probe state machine for one input sample.
    ///
    /// * `sample`      — the raw input sample just popped from the ring buffer.
    /// * `push_output` — closure that pushes one `f32` to the output ring buffer;
    ///                   returns `true` if the push succeeded.
    /// * `probe`       — the shared [`RoundTripProbe`] used to signal results.
    ///
    /// The closure is also responsible for the normal (non-probe) DSP output: when the
    /// probe is idle or counting guard samples the closure is called with the
    /// already-processed `dsp_sample`; when an impulse needs to be injected it is
    /// called with [`IMPULSE_AMPLITUDE`] instead.
    pub fn tick(
        &mut self,
        sample: f32,
        dsp_sample: f32,
        push_output: &mut impl FnMut(f32) -> bool,
        probe: &RoundTripProbe,
    ) {
        match self.phase {
            // --- Phase 1: collect ambient samples → derive threshold ---
            ProbePhase::CalibrationAmbient => {
                if self.feed_ambient_sample(sample) {
                    // Threshold ready — arm the first impulse slot.
                    self.phase = ProbePhase::WaitingForEcho(0);
                }
                push_output(dsp_sample);
            }

            // --- Phase 2: inject impulse / wait for echo ---
            ProbePhase::WaitingForEcho(_) => {
                // Inject the impulse for this slot if we haven't yet.
                if self.impulse_sent_at.is_none() {
                    if push_output(IMPULSE_AMPLITUDE) {
                        self.arm_impulse();
                        let idx = match self.phase { ProbePhase::WaitingForEcho(i) => i + 1, _ => 0 };
                        println!("[RT-PROBE] Impulse {}/{} injected (threshold={:.4}).", idx, IMPULSE_COUNT, self.threshold);
                    }
                } else {
                    // Listen for echo; push normal audio while waiting.
                    if self.check_echo(sample) {
                        let gen = self.generation;
                        if let Some(avg_ms) = self.record_echo() {
                            println!("[RT-PROBE] SUCCESS! Avg round-trip: {:.2} ms", avg_ms);
                            probe.complete_success(gen, avg_ms);
                        }
                        // If more impulses remain, next tick will inject the next one.
                    } else if self.is_timed_out() {
                        println!("[RT-PROBE] TIMEOUT. Echo didn't arrive above threshold {:.4}", self.threshold);
                        let gen = self.generation;
                        probe.complete_error(gen, "Detection timeout".into());
                        self.abort();
                    }
                    push_output(dsp_sample);
                }
            }

            // --- Idle: just pass audio through ---
            ProbePhase::Idle => {
                push_output(dsp_sample);
            }
        }
    }

    /// Call when no input sample is available (ring buffer empty).
    /// Handles the timeout case while the worker is spinning.
    pub fn tick_idle(&mut self, probe: &RoundTripProbe) {
        if matches!(self.phase, ProbePhase::WaitingForEcho(_)) && self.is_timed_out() {
            println!("[RT-PROBE] TIMEOUT (no input samples arriving)");
            let gen = self.generation;
            probe.complete_error(gen, "No input samples".into());
            self.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// RoundTripProbe — shared between the command layer and the worker thread
// ---------------------------------------------------------------------------

pub struct RoundTripProbe {
    state: Mutex<ProbeState>,
    cv: Condvar,
}

impl RoundTripProbe {
    pub fn new() -> Self {
        Self { state: Mutex::new(ProbeState::default()), cv: Condvar::new() }
    }

    /// Request a new measurement. Returns the generation ID to pass to `wait_for_result`.
    pub fn request(&self) -> u64 {
        let mut s = self.state.lock().unwrap();
        s.generation = s.generation.saturating_add(1);
        s.request_pending = true;
        s.result_ms = None;
        s.error = None;
        let gen = s.generation;
        self.cv.notify_all();
        gen
    }

    /// Atomically claim a pending request. Called by the worker thread.
    pub fn try_take_pending_request(&self) -> Option<u64> {
        let mut s = self.state.lock().unwrap();
        if s.request_pending { s.request_pending = false; Some(s.generation) } else { None }
    }

    pub fn complete_success(&self, generation: u64, latency_ms: f64) {
        let mut s = self.state.lock().unwrap();
        if s.generation != generation { return; }
        s.completed_generation = generation;
        s.result_ms = Some(latency_ms);
        s.error = None;
        self.cv.notify_all();
    }

    pub fn complete_error(&self, generation: u64, error: String) {
        let mut s = self.state.lock().unwrap();
        if s.generation != generation { return; }
        s.completed_generation = generation;
        s.result_ms = None;
        s.error = Some(error);
        self.cv.notify_all();
    }

    /// Block until the given generation completes or `timeout` elapses.
    pub fn wait_for_result(&self, generation: u64, timeout: Duration) -> Result<f64, String> {
        let mut s = self.state.lock().unwrap();
        let deadline = Instant::now() + timeout;

        while s.completed_generation < generation {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err("Round-trip latency measurement timed out".to_string());
            }
            let (new_s, timed_out) = self.cv.wait_timeout(s, remaining).unwrap();
            s = new_s;
            if timed_out.timed_out() && s.completed_generation < generation {
                return Err("Round-trip latency measurement timed out".to_string());
            }
        }

        s.result_ms.ok_or_else(|| {
            s.error.clone().unwrap_or_else(|| "Round-trip latency measurement failed".to_string())
        })
    }

    /// Abort any in-flight measurement (e.g. loopback stopped).
    pub fn fail_current(&self, reason: String) {
        let mut s = self.state.lock().unwrap();
        if s.generation == 0 { return; }
        s.request_pending = false;
        s.completed_generation = s.generation;
        s.result_ms = None;
        s.error = Some(reason);
        self.cv.notify_all();
    }
}
