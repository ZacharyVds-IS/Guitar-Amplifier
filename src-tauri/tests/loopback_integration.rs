use ringbuf::HeapRb;
use ringbuf::traits::Split;
use rustriff_lib::services::audioservice::{fill_output_buffer, push_input_samples};

#[test]
fn loopback_pipeline_end_to_end() {
    let rb = HeapRb::<f32>::new(32);
    let (mut prod, mut cons) = rb.split();
    let input_chunk_1 = [0.1, 0.2, 0.3, 0.4];
    let input_chunk_2 = [0.5, 0.6, 0.7, 0.8];

    push_input_samples(&input_chunk_1, &mut prod);
    push_input_samples(&input_chunk_2, &mut prod);

    let mut output = [0.0f32; 8];
    fill_output_buffer(&mut cons, &mut output);

    assert_eq!(
        output,
        [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]
    );

    let mut output2 = [1.0f32; 4];
    fill_output_buffer(&mut cons, &mut output2);
    assert_eq!(output2, [0.0; 4]);
}
