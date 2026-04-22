use std::f32::consts::PI;

pub enum ShelfType {
    Low,
    High,
}

pub struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    z1: f32, z2: f32,
}

impl Biquad {
    pub fn new_shelf(
        shelf: ShelfType,
        sample_rate: f32,
        freq: f32,
        gain_db: f32,
    ) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos = w0.cos();
        let sin = w0.sin();
        let alpha = sin / 2.0 * (2.0 * (a + 1.0 / a)).sqrt();
        let sqrt_a = a.sqrt();

        let (b0, b1, b2, a0, a1, a2) = match shelf {
            ShelfType::Low => (
                a*((a+1.0)-(a-1.0)*cos+2.0*sqrt_a*alpha),
                2.0*a*((a-1.0)-(a+1.0)*cos),
                a*((a+1.0)-(a-1.0)*cos-2.0*sqrt_a*alpha),
                (a+1.0)+(a-1.0)*cos+2.0*sqrt_a*alpha,
                -2.0*((a-1.0)+(a+1.0)*cos),
                (a+1.0)+(a-1.0)*cos-2.0*sqrt_a*alpha,
            ),
            ShelfType::High => (
                a*((a+1.0)+(a-1.0)*cos+2.0*sqrt_a*alpha),
                -2.0*a*((a-1.0)+(a+1.0)*cos),
                a*((a+1.0)+(a-1.0)*cos-2.0*sqrt_a*alpha),
                (a+1.0)-(a-1.0)*cos+2.0*sqrt_a*alpha,
                2.0*((a-1.0)-(a+1.0)*cos),
                (a+1.0)-(a-1.0)*cos-2.0*sqrt_a*alpha,
            ),
        };

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}