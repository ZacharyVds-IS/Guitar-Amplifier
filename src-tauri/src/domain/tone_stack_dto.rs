use serde::{Deserialize, Serialize};

/// Data transfer object for tone stack parameters.
///
/// Used for serializing and deserializing tone stack data, typically for communication between
/// the UI and backend. The bass, middle, and treble values are expected to be in the range 0.0 to 1.0.
#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct ToneStackDto{
    /// The bass level (0.0 to 1.0).
    pub bass: f32,
    /// The middle level (0.0 to 1.0).
    pub middle: f32,
    /// The treble level (0.0 to 1.0).
    pub treble: f32
}