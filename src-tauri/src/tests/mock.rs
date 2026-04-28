use crate::infrastructure::audio_handler::{MockAudioHandlerTrait, PlayableStream};

pub struct FakeStream;

impl PlayableStream for FakeStream {
    fn play(&self) {
    }
}

unsafe impl Send for FakeStream {}

pub fn make_mock_handler() -> MockAudioHandlerTrait {
    make_mock_handler_with_rates(48_000, 48_000)
}

pub fn make_mock_handler_with_rates(input_rate: u32, output_rate: u32) -> MockAudioHandlerTrait {
    let mut mock = MockAudioHandlerTrait::new();

    mock.expect_build_input_stream()
        .returning(|_prod| Box::new(FakeStream));

    mock.expect_build_output_stream()
        .returning(|_cons| Box::new(FakeStream));

    mock.expect_input_sample_rate()
        .return_const(input_rate);

    mock.expect_output_sample_rate()
        .return_const(output_rate);

    mock
}
