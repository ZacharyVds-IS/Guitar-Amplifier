use serde::Serialize;

#[derive(Serialize,Clone)]
pub struct ToneStackDto{
    pub bass: f32,
    pub middle: f32,
    pub treble: f32
}