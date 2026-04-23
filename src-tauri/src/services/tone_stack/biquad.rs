use std::f32::consts::PI;

pub enum ShelfType {
    Low,
    High,
    Peak,
}

pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
    freq: f32,
    shelf_type: ShelfType,
}

// Biquad shelf filter implementation based on RBJ Audio EQ Cookbook
// Reference: https://www.w3.org/2011/audio/audio-eq-cookbook.html
impl Biquad {
    pub fn new_shelf(shelf: ShelfType, sample_rate: f32, freq: f32, gain_db: f32) -> Self {
        let (b0, b1, b2, _a0, a1, a2) =
            Self::calculate_coefficients(&shelf, sample_rate, freq, gain_db);

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
            freq,
            shelf_type: shelf,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn set_gain_db(&mut self, gain_db: f32) {
        let (b0, b1, b2, a0, a1, a2) =
            Self::calculate_coefficients(&self.shelf_type, self.sample_rate, self.freq, gain_db);

        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
        /*
        println!(
            "Updated Biquad Coefficients: b0: {:.6}\t  b1: {:.6}\t  b2: {:.6}\t  a1: {:.6}\t  a2: {:.6}\t  a0: {:.6} (should always be 1.0) || Gain (dB): {:.2}",
            self.b0, self.b1, self.b2, self.a1, self.a2, a0, gain_db
        );
         */
    }

    fn calculate_coefficients(
        shelf: &ShelfType,
        sample_rate: f32,
        freq: f32,
        gain_db: f32,
    ) -> (f32, f32, f32, f32, f32, f32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos = w0.cos();
        let sin = w0.sin();

        let (b0, b1, b2, a0, a1, a2) = match shelf {
            ShelfType::Low | ShelfType::High => {
                let alpha = sin / 16.0 * (2.0 * (a + 1.0 / a)).sqrt();
                let sqrt_a = a.sqrt();
                if matches!(shelf, ShelfType::Low) {
                    (
                        a * ((a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                        2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                        a * ((a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                        (a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                        -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                        (a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
                    )
                } else {
                    (
                        a * ((a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                        -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                        a * ((a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                        (a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                        2.0 * ((a - 1.0) - (a + 1.0) * cos),
                        (a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
                    )
                }
            }
            ShelfType::Peak => {
                let alpha = sin / 8.0;
                (
                    1.0 + alpha * a,
                    -2.0 * cos,
                    1.0 - alpha * a,
                    1.0 + alpha / a,
                    -2.0 * cos,
                    1.0 - alpha / a,
                )
            }
        };
        //Normalize Coefficients so that a0 is 1
        (b0 / a0, b1 / a0, b2 / a0, a0 / a0, a1 / a0, a2 / a0)
    }
}
