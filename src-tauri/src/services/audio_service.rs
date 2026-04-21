use crate::infrastructure::audio_handler::{AudioHandler, AudioHandlerTrait};
use crate::services::gain_processor::GainProcessor;
use cpal::traits::StreamTrait;
use cpal::{Device, StreamConfig};
use std::sync::Arc;
use std::thread;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use crate::domain::audio_processor::AudioProcessor;
use crate::domain::channel::Channel;

pub struct AudioService {
    audio_handler: Arc<dyn AudioHandlerTrait>,
    channel: Channel
}

impl AudioService {
    pub fn new(input_device: Device, output_device: Device, config: StreamConfig) -> Self {
        let handler = AudioHandler::new(input_device, output_device, config);
        Self {
            audio_handler: Arc::new(handler),
            channel: Channel::new("Main".to_string(),None, None),
        }
    }

    pub fn start_loopback(&self) {
        let handler = self.audio_handler.clone();
        let channel = self.channel.clone();

        thread::spawn(move || {
            let (i_producer, mut i_consumer) = AudioHandler::create_ringbuffer(48000);
            let (mut o_producer, o_consumer) = AudioHandler::create_ringbuffer(48000);

            let input_stream = handler.build_input_stream(i_producer);
            let output_stream = handler.build_output_stream(o_consumer);

            thread::spawn(move || {
                let mut gain = GainProcessor::new(channel.gain());
                let mut master_volume = GainProcessor::new(channel.master_volume());

                loop {
                    if let Some(sample) = i_consumer.try_pop() {
                        let sample = gain.process(sample);

                        //Master volume should stay the last alteration to the signal
                        let sample = master_volume.process(sample);
                        let _ = o_producer.try_push(sample);
                    } else {
                        std::thread::yield_now();
                    }
                }
            });

            input_stream.play().unwrap();
            output_stream.play().unwrap();

            thread::park();
        });
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}
