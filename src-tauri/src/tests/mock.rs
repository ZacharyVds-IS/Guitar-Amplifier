use crate::infrastructure::audio_handler::{MockAudioHandlerTrait, PlayableStream};

pub struct FakeStream;

impl PlayableStream for FakeStream {
    fn play(&self) {
    }
}

unsafe impl Send for FakeStream {}

pub fn make_mock_handler() -> MockAudioHandlerTrait {
    let mut mock = MockAudioHandlerTrait::new();

    mock.expect_build_input_stream()
        .returning(|_prod| Box::new(FakeStream));

    mock.expect_build_output_stream()
        .returning(|_cons| Box::new(FakeStream));

    mock
}

