use crate::services::tone_stack::biquad::{Biquad, ShelfType};

pub struct RangeEQ {
    low_shelf: Biquad,
    high_shelf: Biquad,
    sample_rate: f32,
    low_hz: f32,
    high_hz: f32,
}


impl RangeEQ {
    pub fn new(
        sample_rate: f32,
        low_hz: f32,
        high_hz: f32,
        percent: f32,
    ) -> Self {
        let gain_db = percent_to_db(percent);

        Self {
            low_shelf: Biquad::new_shelf(
                ShelfType::High,
                sample_rate,
                low_hz,
                gain_db,
            ),
            high_shelf: Biquad::new_shelf(
                ShelfType::Low,
                sample_rate,
                high_hz,
                gain_db,
            ),
            sample_rate,
            low_hz,
            high_hz,
        }
    }

    pub fn set_percent(&mut self, percent: f32) {
        let gain_db = percent_to_db(percent);

        self.low_shelf = Biquad::new_shelf(
            ShelfType::High,
            self.sample_rate,
            self.low_hz,
            gain_db,
        );
        self.high_shelf = Biquad::new_shelf(
            ShelfType::Low,
            self.sample_rate,
            self.high_hz,
            gain_db,
        );
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let x = self.low_shelf.process(sample);
        self.high_shelf.process(x)
    }
}

fn percent_to_db(percent: f32) -> f32 {
    let p = percent.clamp(0.0001, 1.0);
    20.0 * p.log10()
}
