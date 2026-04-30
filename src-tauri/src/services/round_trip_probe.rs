/// Round-trip latency measurement engine.
///
/// **Calibration** — listens to [`CALIBRATION_SAMPLES`] input samples to measure the
/// ambient noise peak, then derives a detection threshold that is always reachable by
/// the outgoing impulse regardless of input volume.
///
/// **Measurement** — fires [`IMPULSE_COUNT`] impulses in sequence, waits for each echo,
/// and reports the averaged round-trip time in milliseconds.
///
/// The measurement runs inside a [`RoundTripLatencySession`] that opens its own dedicated
/// CPAL streams for the duration of the measurement, fully independent from regular loopback.
use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use cpal::BufferSize;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use std::thread;
use std::time::{Duration, Instant};


// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

const CALIBRATION_SAMPLES: usize = 512;
const IMPULSE_COUNT: usize = 3;
const GUARD_SAMPLES: usize = 512;
const INTER_IMPULSE_GAP: Duration = Duration::from_millis(200);

pub const IMPULSE_AMPLITUDE: f32 = 0.95;

pub enum RoundTripTickOutcome {
    Ongoing,
    Complete(f64),
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundTripMeasurementPhase {
    Idle,
    CalibrationAmbient,
    WaitingForEcho(usize),
}

pub struct RoundTripMeasurementState {
    pub phase: RoundTripMeasurementPhase,
    pub threshold: f32,
    pub impulse_sent_at: Option<Instant>,
    ambient_peak: f32,
    ambient_count: usize,
    guard_remaining: usize,
    echo_durations_ms: Vec<f64>,
    impulse_deadline: Option<Instant>,
    next_impulse_not_before: Option<Instant>,
}

impl RoundTripMeasurementState {
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

    fn feed_ambient_sample(&mut self, sample: f32) -> bool {
        self.ambient_peak = self.ambient_peak.max(sample.abs());
        self.ambient_count += 1;

        if self.ambient_count < CALIBRATION_SAMPLES {
            return false;
        }

        self.threshold = (self.ambient_peak * 2.0)
            .max(0.05)
            .min(IMPULSE_AMPLITUDE * 0.5);

        println!(
            "[RT-MEASURE] Calibration done. Peak: {:.4}, threshold: {:.4}",
            self.ambient_peak, self.threshold
        );
        true
    }

    fn arm_impulse(&mut self, per_impulse_timeout: Duration) {
        let now = Instant::now();
        self.impulse_sent_at = Some(now);
        self.impulse_deadline = Some(now + per_impulse_timeout);
        self.guard_remaining = GUARD_SAMPLES;
    }

    fn check_echo(&mut self, sample: f32) -> bool {
        if self.guard_remaining > 0 {
            self.guard_remaining -= 1;
            return false;
        }
        sample.abs() >= self.threshold
    }

    fn is_timed_out(&self) -> bool {
        self.impulse_deadline
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }

    pub fn tick(
        &mut self,
        sample: f32,
        push_output: &mut impl FnMut(f32) -> bool,
        per_impulse_timeout: Duration,
    ) -> RoundTripTickOutcome {
        match self.phase {
            RoundTripMeasurementPhase::CalibrationAmbient => {
                if self.feed_ambient_sample(sample) {
                    self.phase = RoundTripMeasurementPhase::WaitingForEcho(0);
                }
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
            RoundTripMeasurementPhase::WaitingForEcho(idx) => {
                if self.impulse_sent_at.is_none() {
                    if self
                        .next_impulse_not_before
                        .map(|t| Instant::now() < t)
                        .unwrap_or(false)
                    {
                        push_output(0.0);
                        return RoundTripTickOutcome::Ongoing;
                    }

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
                            let avg = self.echo_durations_ms.iter().sum::<f64>()
                                / self.echo_durations_ms.len() as f64;
                            println!("[RT-MEASURE] Done. Avg round-trip: {:.2} ms", avg);
                            self.phase = RoundTripMeasurementPhase::Idle;
                            RoundTripTickOutcome::Complete(avg)
                        } else {
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
                push_output(0.0);
                RoundTripTickOutcome::Ongoing
            }
        }
    }
}

pub struct RoundTripLatencySession;

impl RoundTripLatencySession {
    pub fn run(
        handler: &dyn AudioHandlerTrait,
        per_impulse_timeout: Duration,
        stream_warmup: Duration,
    ) -> Result<f64, String> {
        fn frames_or_default(buffer_size: BufferSize) -> usize {
            match buffer_size {
                BufferSize::Fixed(frames) => frames as usize,
                BufferSize::Default => 256,
            }
        }

        let configured_frames = frames_or_default(handler.input_config().buffer_size.clone())
            .max(frames_or_default(handler.output_config().buffer_size.clone()));
        let ringbuffer_size = (configured_frames * 4).max(512);

        let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);
        let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(ringbuffer_size);

        let input_stream = handler.build_input_stream(i_producer);
        let output_stream = handler.build_output_stream(o_consumer);
        input_stream.play();
        output_stream.play();

        println!(
            "[RT-MEASURE] Dedicated streams started. Warming up for {stream_warmup:?}..."
        );
        thread::sleep(stream_warmup);

        let mut drained = 0usize;
        while i_consumer.try_pop().is_some() {
            drained += 1;
        }
        println!(
            "[RT-MEASURE] Drained {drained} stale warmup samples. Starting calibration."
        );

        let mut state = RoundTripMeasurementState::new();
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
                thread::yield_now();
            }
        }
    }
}
