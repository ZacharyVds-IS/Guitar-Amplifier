use crate::domain::audio_processor::AudioProcessor;
use crate::domain::tone_stack::ToneStack;
use crate::services::tone_stack::range_eq::RangeEQ;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct ToneStackProcessor {
    tone_stack: Arc<ToneStack>,
    bass_eq: RangeEQ,
    mid_eq: RangeEQ,
    treble_eq: RangeEQ,
}

const BASS_MIN: f32 = 10.0;
const BASS_MAX: f32 = 180.0;
const MID_MAX: f32 = 2_400.0;
const TREBLE_MAX: f32 = 20_000.0;

impl ToneStackProcessor {
    pub fn new(tone_stack: Arc<ToneStack>) -> Self {
        Self {
            tone_stack,
            bass_eq: RangeEQ::new(48000.0, BASS_MIN, BASS_MAX, 1.0),
            mid_eq: RangeEQ::new(48000.0, BASS_MAX, MID_MAX, 1.0),
            treble_eq: RangeEQ::new(48000.0, MID_MAX, TREBLE_MAX, 1.0),
        }
    }

    pub fn print_tone_stack(&self, gain_sample: f32, fft_buffer: &mut Vec<f32>, fft_size: usize) {
        fft_buffer.push(gain_sample);

        if fft_buffer.len() == fft_size {
            let windowed = hann_window(&fft_buffer);

            let spectrum =
                samples_fft_to_spectrum(&windowed, 48_000, FrequencyLimit::All, None).unwrap();

            let mut bass_energy = 0.0f32;
            let mut mid_energy = 0.0f32;
            let mut treble_energy = 0.0f32;

            for (freq, magnitude) in spectrum.data().iter() {
                let f = freq.val();

                if f >= BASS_MIN && f <= BASS_MAX {
                    bass_energy += magnitude.val();
                } else if f > BASS_MAX && f <= MID_MAX {
                    mid_energy += magnitude.val();
                } else if f > MID_MAX && f <= TREBLE_MAX {
                    treble_energy += magnitude.val();
                }
            }

            println!(
                "Tone Stack: Bass: {:>8.5} | Mid: {:>8.5} | Treble: {:>8.5}",
                bass_energy, mid_energy, treble_energy
            );

            fft_buffer.clear();
        }
    }

    fn update_parameters(&mut self) {
        self.bass_eq
            .set_percent(self.tone_stack.bass().load(Ordering::Relaxed));
        self.mid_eq
            .set_percent(self.tone_stack.middle().load(Ordering::Relaxed));
        self.treble_eq
            .set_percent(self.tone_stack.treble().load(Ordering::Relaxed));
    }
}

impl AudioProcessor for ToneStackProcessor {
    fn process(&mut self, sample: f32) -> f32 {
        self.update_parameters();
        let processed = self.bass_eq.process(sample);
        let processed = self.mid_eq.process(processed);
        self.treble_eq.process(processed)
    }
}
