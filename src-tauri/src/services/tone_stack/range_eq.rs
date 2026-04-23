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
