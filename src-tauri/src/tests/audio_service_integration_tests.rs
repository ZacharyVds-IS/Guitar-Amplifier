#[cfg(feature = "test-utils")]
#[test]
fn test_audio_service_toggle_logic() {
    let mock_handler = Arc::new(MockAudioHandler::default());
    let mut service = AudioService::new_with_handler(mock_handler.clone());

    assert!(!service.is_active());

    service.toggle_loopback(true);
    assert!(service.is_active());
    assert!(service.loopback_thread().is_some());

    service.start_loopback();
    assert!(service.is_active());

    service.toggle_loopback(false);
    assert!(!service.is_active());
    assert!(service.loopback_thread().is_none());
}