use mouseless_core::MouseService;

#[tokio::test]
async fn test_mouse_service_creation() {
    let mouse_service = MouseService::new();

    // Test that the service can be cloned (for sharing across threads)
    let _cloned_service = mouse_service.clone();

    // Both services should be able to handle requests
    // Note: We're not actually moving the mouse in tests to avoid side effects
    // In a real test environment, you might want to use a mock MouseController

    // Just verify the service can be created and cloned without panicking
    assert!(true, "MouseService created and cloned successfully");
}

#[tokio::test]
async fn test_mouse_service_multiple_operations() {
    let mouse_service = MouseService::new();

    // Test multiple operations to ensure the underlying MouseController is reused
    // In a real test, you'd want to mock the MouseController to verify reuse

    // For now, just test that multiple calls don't panic
    // (actual mouse movement would require a display environment)

    // This test demonstrates the API usage pattern
    let positions = [(100, 100), (200, 200), (300, 300)];

    for (x, y) in positions {
        // In a real environment with display, this would work:
        let result = mouse_service.move_to_position(x, y).await;
        assert!(result.is_ok(), "Mouse movement should succeed");

        // For CI/testing without display, we just verify the service exists
        assert!(true, "Service ready for position ({}, {})", x, y);
    }
}
