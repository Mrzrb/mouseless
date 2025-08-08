use tracing::{info, debug};

use crate::{
    error::Result,
    models::{KeyInput, Action, Position, AnimationType, MouseButton, ScrollDirection, InteractionMode},
    traits::KeyBindings,
};

/// Basic movement mode implementation
/// Handles I/K/J/L keys for directional cursor movement
/// N/M keys for left/right mouse clicks
/// U/O/Y/P keys for scrolling operations
/// B key for click-and-hold toggle functionality
pub struct BasicMode {
    /// Current movement speed multiplier
    movement_speed: f32,
    
    /// Whether fast mode is enabled
    fast_mode: bool,
    
    /// Current hold state for click-and-hold
    hold_state: bool,
    
    /// Base movement distance in pixels
    base_movement_distance: i32,
    
    /// Base scroll amount
    base_scroll_amount: i32,
}

impl BasicMode {
    /// Create a new basic mode instance
    pub fn new() -> Self {
        Self {
            movement_speed: 1.0,
            fast_mode: false,
            hold_state: false,
            base_movement_distance: 20,
            base_scroll_amount: 3,
        }
    }
    
    /// Set movement speed multiplier
    pub fn set_movement_speed(&mut self, speed: f32) {
        self.movement_speed = speed.max(0.1).min(10.0); // Clamp between 0.1 and 10.0
        debug!("Movement speed set to: {}", self.movement_speed);
    }
    
    /// Get current movement speed
    pub fn get_movement_speed(&self) -> f32 {
        self.movement_speed
    }
    
    /// Toggle fast mode
    pub fn toggle_fast_mode(&mut self) {
        self.fast_mode = !self.fast_mode;
        info!("Fast mode toggled to: {}", self.fast_mode);
    }
    
    /// Check if fast mode is enabled
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode
    }
    
    /// Toggle hold state
    pub fn toggle_hold_state(&mut self) {
        self.hold_state = !self.hold_state;
        info!("Hold state toggled to: {}", self.hold_state);
    }
    
    /// Check if currently in hold state
    pub fn is_holding(&self) -> bool {
        self.hold_state
    }
    
    /// Set hold state explicitly
    pub fn set_hold_state(&mut self, holding: bool) {
        self.hold_state = holding;
        debug!("Hold state set to: {}", holding);
    }
    
    /// Calculate actual movement distance based on current settings
    fn get_movement_distance(&self) -> i32 {
        let speed_multiplier = if self.fast_mode {
            self.movement_speed * 3.0 // Fast mode is 3x faster
        } else {
            self.movement_speed
        };
        
        (self.base_movement_distance as f32 * speed_multiplier) as i32
    }
    
    /// Calculate actual scroll amount based on current settings
    fn get_scroll_amount(&self) -> i32 {
        let speed_multiplier = if self.fast_mode {
            self.movement_speed * 2.0 // Fast mode is 2x faster for scrolling
        } else {
            self.movement_speed
        };
        
        (self.base_scroll_amount as f32 * speed_multiplier) as i32
    }
    
    /// Process input for basic movement mode
    pub fn process_input(&mut self, input: KeyInput, bindings: &KeyBindings) -> Result<Action> {
        let key = input.key;
        
        // Movement keys (I/K/J/L)
        let movement_distance = self.get_movement_distance();
        
        if key == bindings.move_up {
            debug!("Processing move up command");
            return Ok(Action::MoveCursor(
                Position::new(0, -movement_distance),
                AnimationType::Smooth
            ));
        }
        
        if key == bindings.move_down {
            debug!("Processing move down command");
            return Ok(Action::MoveCursor(
                Position::new(0, movement_distance),
                AnimationType::Smooth
            ));
        }
        
        if key == bindings.move_left {
            debug!("Processing move left command");
            return Ok(Action::MoveCursor(
                Position::new(-movement_distance, 0),
                AnimationType::Smooth
            ));
        }
        
        if key == bindings.move_right {
            debug!("Processing move right command");
            return Ok(Action::MoveCursor(
                Position::new(movement_distance, 0),
                AnimationType::Smooth
            ));
        }
        
        // Click keys (N/M)
        if key == bindings.left_click {
            debug!("Processing left click command");
            return Ok(Action::Click(MouseButton::Left));
        }
        
        if key == bindings.right_click {
            debug!("Processing right click command");
            return Ok(Action::Click(MouseButton::Right));
        }
        
        // Middle click if configured
        if let Some(middle_key) = bindings.middle_click {
            if key == middle_key {
                debug!("Processing middle click command");
                return Ok(Action::Click(MouseButton::Middle));
            }
        }
        
        // Scroll keys (U/O/Y/P)
        let scroll_amount = self.get_scroll_amount();
        
        if key == bindings.scroll_up {
            debug!("Processing scroll up command");
            return Ok(Action::Scroll(ScrollDirection::Up, scroll_amount));
        }
        
        if key == bindings.scroll_down {
            debug!("Processing scroll down command");
            return Ok(Action::Scroll(ScrollDirection::Down, scroll_amount));
        }
        
        if key == bindings.scroll_left {
            debug!("Processing scroll left command");
            return Ok(Action::Scroll(ScrollDirection::Left, scroll_amount));
        }
        
        if key == bindings.scroll_right {
            debug!("Processing scroll right command");
            return Ok(Action::Scroll(ScrollDirection::Right, scroll_amount));
        }
        
        // Hold toggle key (B)
        if key == bindings.hold_toggle {
            debug!("Processing hold toggle command");
            self.toggle_hold_state();
            return Ok(Action::NoAction);
        }
        
        // Speed toggle key (F)
        if key == bindings.speed_toggle {
            debug!("Processing speed toggle command");
            self.toggle_fast_mode();
            return Ok(Action::ToggleSpeed);
        }
        
        // Screen switching keys (1/2/3)
        if key == bindings.screen_1 {
            debug!("Processing screen 1 switch command");
            return Ok(Action::MoveCursor(
                Position::with_screen(0, 0, 1),
                AnimationType::Smooth
            ));
        }
        
        if key == bindings.screen_2 {
            debug!("Processing screen 2 switch command");
            return Ok(Action::MoveCursor(
                Position::with_screen(0, 0, 2),
                AnimationType::Smooth
            ));
        }
        
        if key == bindings.screen_3 {
            debug!("Processing screen 3 switch command");
            return Ok(Action::MoveCursor(
                Position::with_screen(0, 0, 3),
                AnimationType::Smooth
            ));
        }
        
        // Mode switching keys
        if key == bindings.grid_mode {
            debug!("Processing grid mode activation command");
            return Ok(Action::ActivateMode(InteractionMode::Grid));
        }
        
        if key == bindings.area_mode {
            debug!("Processing area mode activation command");
            return Ok(Action::ActivateMode(InteractionMode::Area));
        }
        
        if key == bindings.prediction_mode {
            debug!("Processing prediction mode activation command");
            return Ok(Action::ActivateMode(InteractionMode::Prediction));
        }
        
        // Exit key
        if key == bindings.exit_key {
            debug!("Processing exit command");
            return Ok(Action::Exit);
        }
        
        // No action for unrecognized keys
        debug!("Unrecognized key in basic mode: '{}'", key);
        Ok(Action::NoAction)
    }
    
    /// Reset the mode to default state
    pub fn reset(&mut self) {
        self.movement_speed = 1.0;
        self.fast_mode = false;
        self.hold_state = false;
        info!("Basic mode reset to default state");
    }
    
    /// Get current mode state as a string for debugging
    pub fn get_state_info(&self) -> String {
        format!(
            "BasicMode {{ speed: {:.1}, fast: {}, holding: {} }}",
            self.movement_speed, self.fast_mode, self.hold_state
        )
    }
}

impl Default for BasicMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    
    fn create_test_input(key: char) -> KeyInput {
        KeyInput {
            key,
            modifiers: vec![],
            timestamp: SystemTime::now(),
        }
    }
    
    #[test]
    fn test_basic_mode_creation() {
        let mode = BasicMode::new();
        assert_eq!(mode.get_movement_speed(), 1.0);
        assert!(!mode.is_fast_mode());
        assert!(!mode.is_holding());
    }
    
    #[test]
    fn test_movement_keys() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        // Test up movement
        let action = mode.process_input(create_test_input('i'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, 0);
                assert_eq!(pos.y, -20); // Moving up
            }
            _ => panic!("Expected MoveCursor action for up movement"),
        }
        
        // Test down movement
        let action = mode.process_input(create_test_input('k'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, 0);
                assert_eq!(pos.y, 20); // Moving down
            }
            _ => panic!("Expected MoveCursor action for down movement"),
        }
        
        // Test left movement
        let action = mode.process_input(create_test_input('j'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, -20); // Moving left
                assert_eq!(pos.y, 0);
            }
            _ => panic!("Expected MoveCursor action for left movement"),
        }
        
        // Test right movement
        let action = mode.process_input(create_test_input('l'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, 20); // Moving right
                assert_eq!(pos.y, 0);
            }
            _ => panic!("Expected MoveCursor action for right movement"),
        }
    }
    
    #[test]
    fn test_click_keys() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        // Test left click
        let action = mode.process_input(create_test_input('n'), &bindings).unwrap();
        assert_eq!(action, Action::Click(MouseButton::Left));
        
        // Test right click
        let action = mode.process_input(create_test_input('m'), &bindings).unwrap();
        assert_eq!(action, Action::Click(MouseButton::Right));
        
        // Test middle click
        let action = mode.process_input(create_test_input(','), &bindings).unwrap();
        assert_eq!(action, Action::Click(MouseButton::Middle));
    }
    
    #[test]
    fn test_scroll_keys() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        // Test scroll up
        let action = mode.process_input(create_test_input('u'), &bindings).unwrap();
        assert_eq!(action, Action::Scroll(ScrollDirection::Up, 3));
        
        // Test scroll down
        let action = mode.process_input(create_test_input('o'), &bindings).unwrap();
        assert_eq!(action, Action::Scroll(ScrollDirection::Down, 3));
        
        // Test scroll left
        let action = mode.process_input(create_test_input('y'), &bindings).unwrap();
        assert_eq!(action, Action::Scroll(ScrollDirection::Left, 3));
        
        // Test scroll right
        let action = mode.process_input(create_test_input('p'), &bindings).unwrap();
        assert_eq!(action, Action::Scroll(ScrollDirection::Right, 3));
    }
    
    #[test]
    fn test_hold_toggle() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        assert!(!mode.is_holding());
        
        // Toggle hold state
        let action = mode.process_input(create_test_input('b'), &bindings).unwrap();
        assert_eq!(action, Action::NoAction);
        assert!(mode.is_holding());
        
        // Toggle again
        let action = mode.process_input(create_test_input('b'), &bindings).unwrap();
        assert_eq!(action, Action::NoAction);
        assert!(!mode.is_holding());
    }
    
    #[test]
    fn test_speed_toggle() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        assert!(!mode.is_fast_mode());
        
        // Toggle speed
        let action = mode.process_input(create_test_input('f'), &bindings).unwrap();
        assert_eq!(action, Action::ToggleSpeed);
        assert!(mode.is_fast_mode());
        
        // Test that movement distance changes in fast mode
        let action = mode.process_input(create_test_input('i'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.y, -60); // 3x faster in fast mode
            }
            _ => panic!("Expected MoveCursor action"),
        }
    }
    
    #[test]
    fn test_screen_switching() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        // Test screen 1
        let action = mode.process_input(create_test_input('1'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.screen_id, Some(1));
            }
            _ => panic!("Expected MoveCursor action for screen switch"),
        }
        
        // Test screen 2
        let action = mode.process_input(create_test_input('2'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.screen_id, Some(2));
            }
            _ => panic!("Expected MoveCursor action for screen switch"),
        }
    }
    
    #[test]
    fn test_mode_switching() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        // Test grid mode activation
        let action = mode.process_input(create_test_input('g'), &bindings).unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Grid));
        
        // Test area mode activation
        let action = mode.process_input(create_test_input('a'), &bindings).unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Area));
        
        // Test prediction mode activation
        let action = mode.process_input(create_test_input('r'), &bindings).unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Prediction));
    }
    
    #[test]
    fn test_exit_key() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        let action = mode.process_input(create_test_input(' '), &bindings).unwrap();
        assert_eq!(action, Action::Exit);
    }
    
    #[test]
    fn test_movement_speed_setting() {
        let mut mode = BasicMode::new();
        
        mode.set_movement_speed(2.0);
        assert_eq!(mode.get_movement_speed(), 2.0);
        
        // Test clamping
        mode.set_movement_speed(15.0);
        assert_eq!(mode.get_movement_speed(), 10.0); // Clamped to max
        
        mode.set_movement_speed(0.05);
        assert_eq!(mode.get_movement_speed(), 0.1); // Clamped to min
    }
    
    #[test]
    fn test_custom_movement_speed() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        mode.set_movement_speed(2.0);
        
        let action = mode.process_input(create_test_input('i'), &bindings).unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.y, -40); // 2x speed
            }
            _ => panic!("Expected MoveCursor action"),
        }
    }
    
    #[test]
    fn test_reset() {
        let mut mode = BasicMode::new();
        
        mode.set_movement_speed(2.0);
        mode.toggle_fast_mode();
        mode.toggle_hold_state();
        
        assert_eq!(mode.get_movement_speed(), 2.0);
        assert!(mode.is_fast_mode());
        assert!(mode.is_holding());
        
        mode.reset();
        
        assert_eq!(mode.get_movement_speed(), 1.0);
        assert!(!mode.is_fast_mode());
        assert!(!mode.is_holding());
    }
    
    #[test]
    fn test_unrecognized_key() {
        let mut mode = BasicMode::new();
        let bindings = KeyBindings::default();
        
        let action = mode.process_input(create_test_input('x'), &bindings).unwrap();
        assert_eq!(action, Action::NoAction);
    }
    
    #[test]
    fn test_state_info() {
        let mode = BasicMode::new();
        let info = mode.get_state_info();
        assert!(info.contains("speed: 1.0"));
        assert!(info.contains("fast: false"));
        assert!(info.contains("holding: false"));
    }
}