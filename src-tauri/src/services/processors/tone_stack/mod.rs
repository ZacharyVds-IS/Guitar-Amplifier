/// Tone stack service module.
///
/// This module provides audio processing components for tone stack equalization,
/// including biquad filters, range EQs, and the main tone stack processor.
/// It implements low-latency audio filtering for bass, middle, and treble adjustments.
pub mod tone_stack_processor;
pub mod biquad;
mod range_eq;