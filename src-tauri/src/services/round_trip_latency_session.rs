//! Round-trip latency measurement using dedicated CPAL streams.
//!
//! This module measures full end-to-end audio latency by:
//! 1. opening temporary input/output streams,
//! 2. listening to ambient input noise to derive a safe detection threshold,
//! 3. emitting a short impulse to the output,
//! 4. waiting for that impulse to return on the input,
//! 5. repeating the process a few times and averaging the results.
//!
//! The logic is intentionally isolated from the normal loopback path so the measurement
//! can run whether or not the main audio engine is currently active.
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use cpal::BufferSize;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::thread;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

// Number of input samples used to estimate the ambient noise floor before the first impulse.
const CALIBRATION_SAMPLES: usize = 512; // ~11 ms @ 44_100 Hz
// Number of impulses to send per measurement; the final result is their average.
const IMPULSE_COUNT: usize = 3;
// Number of samples to ignore immediately after sending an impulse so we do not detect the
// outgoing impulse itself or very early bleed as a valid return signal.
const GUARD_SAMPLES: usize = 512; // ~11 ms @ 44_100 Hz
// Quiet time between measurements so the previous response can decay before the next impulse.
const INTER_IMPULSE_GAP: Duration = Duration::from_millis(200);

/// Peak amplitude used for the synthetic round-trip test impulse.
///
/// The return threshold is clamped relative to this value so the detector remains reachable
/// even when the input path is quiet.
pub const IMPULSE_AMPLITUDE: f32 = 0.95;

/// Result of processing a single input sample in the measurement state machine.
pub enum RoundTripTickOutcome {
    /// The measurement is still running and needs more input samples.
    Ongoing,
    /// All impulses were observed successfully; contains the averaged latency in ms.
    Complete(f64),
    /// The current impulse was never detected before its timeout expired.
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// High-level phases of the round-trip measurement state machine.
pub enum RoundTripMeasurementPhase {
    /// Measurement has finished and no further impulses should be emitted.
    Idle,
    /// Ambient input is being sampled to derive a detection threshold.
    CalibrationAmbient,
    /// An impulse index is active and we are waiting to detect its return signal.
    WaitingForEcho(usize),
}

/// Per-measurement state owned by the dedicated round-trip session thread.
///
/// This type contains all transient information needed to run the calibration/impulse/echo
/// cycle. It has no shared ownership or locking because it lives entirely inside one session.
pub struct RoundTripMeasurementState {
    /// Current phase in the calibration/measurement lifecycle.
    pub phase: RoundTripMeasurementPhase,
    /// Dynamically derived absolute input amplitude required to accept an echo.
    pub threshold: f32,
    /// Timestamp recorded when the current impulse was sent.
    pub impulse_sent_at: Option<Instant>,
    /// Highest absolute ambient sample observed during calibration.
    ambient_peak: f32,
    /// Number of ambient samples consumed so far.
    ambient_count: usize,
    /// Guard counter used to suppress very early post-impulse detection.
    guard_remaining: usize,
    /// Measured latency of each successful impulse/echo pair in milliseconds.
    echo_durations_ms: Vec<f64>,
    /// Timeout deadline for the currently active impulse.
    impulse_deadline: Option<Instant>,
    /// Earliest time at which the next impulse may be sent.
    next_impulse_not_before: Option<Instant>,
}

impl RoundTripMeasurementState {
    /// Create a fresh round-trip measurement state starting in ambient calibration mode.
    pub fn new() -> Self {
        Self {
            phase: RoundTripMeasurementPhase::CalibrationAmbient,
            threshold: 0.0,
            impulse_sent_at: None,
            ambient_peak: 0.0,
            ambient_count: 0,
            guard_remaining: 0,
            echo_durations_ms: Vec::with_capacity(IMPULSE_COUNT),
            impulse_deadline: None,
            next_impulse_not_before: None,
        }
    }

    // Consumes ambient input samples until enough data has been collected to derive a stable
    // threshold above the observed noise floor.
    fn feed_ambient_sample(&mut self, sample: f32) -> bool {
        self.ambient_peak = self.ambient_peak.max(sample.abs());
        self.ambient_count += 1;

        if self.ambient_count < CALIBRATION_SAMPLES {
            return false;
        }

        // The threshold is set above the measured ambient peak, but never below a minimum floor
        // and never so high that a returned impulse becomes impossible to detect.
        self.threshold = (self.ambient_peak * 2.0)
            .max(0.05)
            .min(IMPULSE_AMPLITUDE * 0.5);

        println!(
            "[RT-MEASURE] Calibration done. Peak: {:.4}, threshold: {:.4}",
            self.ambient_peak, self.threshold
        );
        true
    }

    // Starts the timer and guard window for a newly emitted impulse.
    fn arm_impulse(&mut self, per_impulse_timeout: Duration) {
        let now = Instant::now();
        self.impulse_sent_at = Some(now);
        self.impulse_deadline = Some(now + per_impulse_timeout);
        self.guard_remaining = GUARD_SAMPLES;
    }

    // Returns true only when the current sample exceeds the derived threshold and the initial
    // post-impulse guard period has elapsed.
    fn check_echo(&mut self, sample: f32) -> bool {
        if self.guard_remaining > 0 {
            self.guard_remaining -= 1;
            return false;
        }
        sample.abs() >= self.threshold
    }

    // Checks whether the currently active impulse has exceeded its allowed wait time.
    fn is_timed_out(&self) -> bool {
        self.impulse_deadline
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }

    /// Advance the round-trip measurement by one input sample.
    ///
    /// The caller provides:
    /// - `sample`: one captured input sample,
    /// - `push_output`: a closure used to emit either silence or an impulse,
    /// - `per_impulse_timeout`: the maximum wait time for an echo once an impulse was emitted.
    ///
    /// This function drives the full state machine:
    /// - calibrating the threshold,
    /// - emitting impulses,
    /// - ignoring the immediate post-impulse guard window,
    /// - detecting returns,
    /// - averaging the result.
    pub fn tick(
        &mut self,
        sample: f32,
        push_output: &mut impl FnMut(f32) -> bool,
        per_impulse_timeout: Duration,
    ) -> RoundTripTickOutcome {
        match self.phase {
            RoundTripMeasurementPhase::CalibrationAmbient => {
                if self.feed_ambient_sample(sample) {
                    // Calibration is complete; the next tick will be allowed to emit impulse 1.
                    self.phase = RoundTripMeasurementPhase::WaitingForEcho(0);
                }
                // Stay silent during calibration so the measurement does not feed input back out.
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
            RoundTripMeasurementPhase::WaitingForEcho(idx) => {
                if self.impulse_sent_at.is_none() {
                    // Enforce spacing between impulses so one return tail cannot contaminate the next.
                    if self
                        .next_impulse_not_before
                        .map(|t| Instant::now() < t)
                        .unwrap_or(false)
                    {
                        push_output(0.0);
                        return RoundTripTickOutcome::Ongoing;
                    }

                    // Emit exactly one impulse and start timing from that point forward.
                    if push_output(IMPULSE_AMPLITUDE) {
                        self.arm_impulse(per_impulse_timeout);
                        println!(
                            "[RT-MEASURE] Impulse {}/{} injected (threshold={:.4}).",
                            idx + 1,
                            IMPULSE_COUNT,
                            self.threshold
                        );
                    }

                    RoundTripTickOutcome::Ongoing
                } else {
                    // Stay silent while listening for the return signal to avoid creating a
                    // measurement-distorting feedback loop.
                    push_output(0.0);

                    if self.check_echo(sample) {
                        let elapsed_ms = self
                            .impulse_sent_at
                            .take()
                            .unwrap()
                            .elapsed()
                            .as_secs_f64()
                            * 1000.0;
                        self.impulse_deadline = None;
                        self.echo_durations_ms.push(elapsed_ms);

                        println!(
                            "[RT-MEASURE] Echo {}/{}: {:.2} ms",
                            idx + 1,
                            IMPULSE_COUNT,
                            elapsed_ms
                        );

                        if self.echo_durations_ms.len() >= IMPULSE_COUNT {
                            // Use the average to smooth out one-off jitter between callbacks.
                            let avg = self.echo_durations_ms.iter().sum::<f64>()
                                / self.echo_durations_ms.len() as f64;
                            println!("[RT-MEASURE] Done. Avg round-trip: {:.2} ms", avg);
                            self.phase = RoundTripMeasurementPhase::Idle;
                            RoundTripTickOutcome::Complete(avg)
                        } else {
                            // Prepare the next impulse after a short quiet gap.
                            self.next_impulse_not_before = Some(Instant::now() + INTER_IMPULSE_GAP);
                            self.phase = RoundTripMeasurementPhase::WaitingForEcho(idx + 1);
                            RoundTripTickOutcome::Ongoing
                        }
                    } else if self.is_timed_out() {
                        println!(
                            "[RT-MEASURE] TIMEOUT waiting for echo {} (threshold={:.4}).",
                            idx + 1,
                            self.threshold
                        );
                        self.phase = RoundTripMeasurementPhase::Idle;
                        RoundTripTickOutcome::TimedOut
                    } else {
                        RoundTripTickOutcome::Ongoing
                    }
                }
            }
            RoundTripMeasurementPhase::Idle => {
                // Once complete, continue writing silence until the session exits.
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
        }
    }
}

/// Self-contained round-trip latency measurement session.
///
/// A session owns temporary CPAL streams and tears them down automatically when the
/// measurement completes or fails.
pub struct RoundTripLatencySession;

impl RoundTripLatencySession {
    /// Run a full round-trip latency measurement.
    ///
    /// # Arguments
    /// - `handler`: audio I/O factory providing devices and stream configurations.
    /// - `per_impulse_timeout`: maximum wait time for each emitted impulse.
    /// - `stream_warmup`: startup delay that allows newly opened streams to stabilise before
    ///   measurement begins.
    ///
    /// # Returns
    /// - `Ok(latency_ms)` with the averaged round-trip latency.
    /// - `Err(...)` if no valid return signal was detected before timeout.
    pub fn run(
        handler: &dyn AudioHandlerTrait,
        per_impulse_timeout: Duration,
        stream_warmup: Duration,
    ) -> Result<f64, String> {
        // Convert CPAL buffer-size configuration into a usable frame count for our temporary
        // ring buffers. Default mode falls back to a practical, conservative size.
        fn frames_or_default(buffer_size: BufferSize) -> usize {
            match buffer_size {
                BufferSize::Fixed(frames) => frames as usize,
                BufferSize::Default => 256,
            }
        }

        // Size ring buffers relative to the configured hardware buffers so startup and warmup
        // traffic do not immediately overflow them.
        let configured_frames = frames_or_default(handler.input_config().buffer_size.clone())
            .max(frames_or_default(handler.output_config().buffer_size.clone()));
        let ringbuffer_size = (configured_frames * 4).max(512);

        let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
        let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

        let input_stream = handler.build_input_stream(i_producer);
        let output_stream = handler.build_output_stream(o_consumer);
        input_stream.play();
        output_stream.play();

        // Let the backend/device stack settle before starting calibration.
        println!(
            "[RT-MEASURE] Dedicated streams started. Warming up for {stream_warmup:?}..."
        );
        thread::sleep(stream_warmup);

        // Discard startup samples collected during warmup so timing starts from fresh data only.
        let mut drained = 0usize;
        while i_consumer.try_pop().is_some() {
            drained += 1;
        }
        println!(
            "[RT-MEASURE] Drained {drained} stale warmup samples. Starting calibration."
        );

        let mut state = RoundTripMeasurementState::new();
        // The full measurement may include several impulses, so the overall deadline is larger
        // than a single-impulse timeout.
        let overall_deadline =
            Instant::now() + per_impulse_timeout * IMPULSE_COUNT as u32 + Duration::from_secs(2);

        loop {
            if Instant::now() >= overall_deadline {
                return Err("Round-trip measurement timed out (no echo received).".to_string());
            }

            if let Some(sample) = i_consumer.try_pop() {
                match state.tick(sample, &mut |v| o_producer.try_push(v).is_ok(), per_impulse_timeout)
                {
                    RoundTripTickOutcome::Complete(avg_ms) => return Ok(avg_ms),
                    RoundTripTickOutcome::TimedOut => {
                        return Err(format!(
                            "Echo not detected above threshold {:.4}. Ensure output is physically routed back into input.",
                            state.threshold
                        ))
                    }
                    RoundTripTickOutcome::Ongoing => {}
                }
            } else {
                // Cooperatively yield while waiting for the input callback to deliver more data.
                thread::yield_now();
            }
        }
    }
}

