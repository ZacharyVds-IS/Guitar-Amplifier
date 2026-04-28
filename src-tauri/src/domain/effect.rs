use crate::domain::audio_processor::AudioProcessor;

pub trait Effect: AudioProcessor {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);

    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active() {
            self.process(sample)
        } else {
            sample
        }
    }
}