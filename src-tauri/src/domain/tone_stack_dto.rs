use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct ToneStackDto{
    pub bass: f32,
    pub middle: f32,
    pub treble: f32
}