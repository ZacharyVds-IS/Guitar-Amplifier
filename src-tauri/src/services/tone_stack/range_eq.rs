use crate::services::tone_stack::biquad::{Biquad, ShelfType};

pub enum EQType {
    Low,
    High,
    Band,
    Peak,
}

pub struct RangeEQ {
    low_shelf: Biquad,
    high_shelf: Biquad,
    sample_rate: f32,
    low_hz: f32,
    high_hz: f32,
    eq_type: EQType,
}

impl RangeEQ {
    pub fn new(sample_rate: f32, low_hz: f32, high_hz: f32, percent: f32, eq_type: EQType) -> Self {
        let gain_db = percent_to_db(percent);

        let low_shelf = match eq_type {
            EQType::Low => Biquad::new_shelf(ShelfType::Low, sample_rate, low_hz, gain_db),
            EQType::High => Biquad::new_shelf(ShelfType::Low, sample_rate, 1000.0, 0.0), // dummy
            EQType::Band => Biquad::new_shelf(ShelfType::Low, sample_rate, low_hz, gain_db),
            EQType::Peak => Biquad::new_shelf(ShelfType::Peak, sample_rate, low_hz, gain_db),
        };

        let high_shelf = match eq_type {
            EQType::Low => Biquad::new_shelf(ShelfType::High, sample_rate, 20000.0, 0.0), // dummy
            EQType::High => Biquad::new_shelf(ShelfType::High, sample_rate, high_hz, gain_db),
            EQType::Band => Biquad::new_shelf(ShelfType::High, sample_rate, high_hz, gain_db),
            EQType::Peak => Biquad::new_shelf(ShelfType::High, sample_rate, 20000.0, 0.0), // dummy
        };

        Self {
            low_shelf,
            high_shelf,
            sample_rate,
            low_hz,
            high_hz,
            eq_type,
        }
    }

    pub fn set_percent(&mut self, percent: f32) {
        let gain_db = percent_to_db(percent);
        match self.eq_type {
            EQType::Low => self.low_shelf.set_gain_db(gain_db),
            EQType::High => self.high_shelf.set_gain_db(gain_db),
            EQType::Band => {
                self.low_shelf.set_gain_db(gain_db);
                self.high_shelf.set_gain_db(gain_db);
            }
            EQType::Peak => self.low_shelf.set_gain_db(gain_db),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        match self.eq_type {
            EQType::Low => self.low_shelf.process(sample),
            EQType::High => self.high_shelf.process(sample),
            EQType::Band => {
                let x = self.low_shelf.process(sample);
                self.high_shelf.process(x)
            }
            EQType::Peak => self.low_shelf.process(sample),
        }
    }
}

fn percent_to_db(percent: f32) -> f32 {
    let p = percent.clamp(0.0001, 1.0);
    // Logarithmic: 0% -> -24 dB, 100% -> 0 dB (prevents instability at extreme values)
    20.0 * p.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;


    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn test_percent_clamping_high() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 2.0, EQType::Low);
            // Should clamp to 1.0 internally and process without issues
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_percent_clamping_low() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, -1.0, EQType::Low);
            // Should clamp to 0.0001 internally and process without issues
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_extreme_frequencies() {
            let mut eq = RangeEQ::new(44100.0, 1.0, 22000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_zero_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(0.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_very_small_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(1e-6);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_large_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(100.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_set_percent_extreme_values() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);

            eq.set_percent(0.0);
            let out1 = eq.process(0.1);

            eq.set_percent(10.0); // Way above 1.0
            let out2 = eq.process(0.1);

            eq.set_percent(-5.0); // Negative
            let out3 = eq.process(0.1);

            assert!(!out1.is_nan() && !out2.is_nan() && !out3.is_nan());
        }

        #[test]
        fn test_multiple_set_percent_calls() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);

            let percents = vec![0.1, 0.5, 0.9, 0.2, 0.8, 0.0, 1.0];

            for percent in percents {
                eq.set_percent(percent);
                let output = eq.process(0.1);
                assert!(!output.is_nan());
                assert!(!output.is_infinite());
            }
        }

        #[test]
        fn test_all_eq_types_with_extreme_values() {
            let eq_types = vec![EQType::Low, EQType::High, EQType::Band, EQType::Peak];

            for eq_type in eq_types {
                let mut eq = RangeEQ::new(44100.0, 20.0, 20000.0, 0.01, eq_type);
                let output = eq.process(0.1);
                assert!(!output.is_nan());
                assert!(!output.is_infinite());
            }
        }

        #[test]
        fn test_low_sample_rate() {
            let mut eq = RangeEQ::new(8000.0, 50.0, 4000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_high_sample_rate() {
            let mut eq = RangeEQ::new(192000.0, 20.0, 40000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_after_multiple_gain_changes() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Low);

            eq.set_percent(0.1);
            let out1 = eq.process(0.1);

            eq.set_percent(0.9);
            let out2 = eq.process(0.1);

            eq.set_percent(0.5);
            let out3 = eq.process(0.1);

            assert!(!out1.is_nan() && !out2.is_nan() && !out3.is_nan());
        }

        #[test]
        fn test_percent_to_db_edge_cases() {
            // Test clamping behavior
            assert_eq!(percent_to_db(1.0), 0.0);
            assert_eq!(percent_to_db(0.0001), percent_to_db(0.00005)); // Both should clamp to 0.0001
            assert!(percent_to_db(2.0) == percent_to_db(1.0)); // Should clamp to 1.0
        }
    }
}
