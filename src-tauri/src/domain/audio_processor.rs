pub trait AudioProcessor: Send {
    fn process(&mut self, sample: f32) ->f32;
}