#[cfg(test)]
mod integration_tests {
    use crate::{
        ModeManager, KeyBindings, InteractionMode, KeyInput, Action, 
        MouseButton, AnimationType, ModeEvent,
        traits::ModeController
    };
    use std::time::SystemTime;
    
    fn create_test_input(key: char) -> KeyInput {
        KeyInput {
            key,
            modifiers: vec![],
            timestamp: SystemTime::now(),
        }
    }
    
    // Note: This test was removed due to potential async runtime issues
    // The functionality is covered by individual unit tests in basic_mode and mode modules
    
    #[test]
    fn test_mode_manager_event_system() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut manager = ModeManager::new(KeyBindings::default());
            let mut receiver = manager.subscribe_to_events();
            
            // Activate a mode and check for event
            manager.activate_mode(InteractionMode::Basic).await.unwrap();
            
            let event = receiver.try_recv().unwrap();
            assert_eq!(event, ModeEvent::ModeActivated(InteractionMode::Basic));
            
            // Switch to another mode and check for event
            manager.activate_mode(InteractionMode::Grid).await.unwrap();
            
            let event = receiver.try_recv().unwrap();
            assert_eq!(event, ModeEvent::ModeChanged { 
                from: InteractionMode::Basic, 
                to: InteractionMode::Grid 
            });
            
            // Deactivate and check for event
            manager.deactivate_current_mode().await.unwrap();
            
            let event = receiver.try_recv().unwrap();
            assert_eq!(event, ModeEvent::ModeDeactivated(InteractionMode::Grid));
        });
    }
    
    #[test]
    fn test_mode_manager_state_synchronization() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = ModeManager::new(KeyBindings::default());
            
            // Test movement speed synchronization
            manager.set_movement_speed(2.5);
            assert_eq!(manager.get_movement_speed(), 2.5);
            
            // Test hold state synchronization
            assert!(!manager.is_holding());
            manager.set_hold_state(true);
            assert!(manager.is_holding());
            
            // Test speed toggle synchronization
            manager.toggle_speed();
            // The internal fast_mode should be toggled
            
            // Test that basic mode input processing uses the synchronized state
            let action = manager.handle_input(create_test_input('i')).await.unwrap();
            match action {
                Action::MoveCursor(pos, _) => {
                    // Should be 2.5 * 3 (fast mode) * 20 (base distance) = 150
                    assert_eq!(pos.y, -150);
                }
                _ => panic!("Expected MoveCursor action"),
            }
        });
    }
    
    #[test]
    fn test_mode_manager_history_tracking() {
        let mut manager = ModeManager::new(KeyBindings::default());
        
        // Initially no history
        assert_eq!(manager.get_mode_history().len(), 0);
        
        // Add some modes to history by activating them
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.activate_mode(InteractionMode::Basic).await.unwrap();
            manager.activate_mode(InteractionMode::Grid).await.unwrap();
            manager.activate_mode(InteractionMode::Area).await.unwrap();
        });
        
        let history = manager.get_mode_history();
        assert_eq!(history.len(), 2); // Grid and Basic should be in history
        assert_eq!(history[0], InteractionMode::Grid); // Most recent first
        assert_eq!(history[1], InteractionMode::Basic);
    }
}