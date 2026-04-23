pub trait AudioProcessor: Send {
    /// Processes a single audio sample and returns the processed result.
    ///
    /// # Arguments
    ///
    /// * `sample` - A normalized `f32` audio sample (typically -1.0 to 1.0).
    ///
    /// # Returns
    ///
    /// The processed audio sample.
    fn process(&mut self, sample: f32) -> f32;
}