use std::collections::HashMap;
use tracing::{debug, info};

use crate::{
    error::Result,
    models::{Action, AnimationType, Area, InteractionMode, KeyInput, Position, ScreenBounds},
    traits::KeyBindings,
};

/// Area mode handler for 9-region screen navigation
pub struct AreaMode {
    /// Whether area mode is currently active
    is_active: bool,

    /// Screen areas mapped to their keys
    areas: HashMap<char, Area>,

    /// Screen dimensions for area calculation
    screen_width: f64,
    screen_height: f64,

    /// Current combination state for Q+E style navigation
    first_key: Option<char>,

    /// Timestamp of first key press for timeout handling
    first_key_time: Option<std::time::Instant>,
}

impl AreaMode {
    /// Create a new area mode handler
    pub fn new() -> Self {
        Self {
            is_active: false,
            areas: HashMap::new(),
            screen_width: 1920.0,  // Default, will be updated
            screen_height: 1080.0, // Default, will be updated
            first_key: None,
            first_key_time: None,
        }
    }

    /// Set screen dimensions and recalculate areas
    pub fn set_screen_dimensions(&mut self, width: f64, height: f64) {
        self.screen_width = width;
        self.screen_height = height;
        self.calculate_areas();
        debug!("Screen dimensions updated: {}x{}", width, height);
    }

    /// Activate area mode
    pub fn activate(&mut self) {
        self.is_active = true;
        self.reset_combination_state();
        self.calculate_areas();
        info!("Area mode activated");
    }

    /// Deactivate area mode
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.reset_combination_state();
        info!("Area mode deactivated");
    }

    /// Check if area mode is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get current areas for UI rendering
    pub fn get_areas(&self) -> Vec<Area> {
        self.areas.values().cloned().collect()
    }

    /// Reset combination state
    fn reset_combination_state(&mut self) {
        self.first_key = None;
        self.first_key_time = None;
    }

    /// Calculate 9 screen areas based on current dimensions
    fn calculate_areas(&mut self) {
        self.areas.clear();

        let area_width = self.screen_width / 3.0;
        let area_height = self.screen_height / 3.0;

        // Define the 9 areas with their keys (Q/W/E/A/S/D/Z/X/C)
        let area_keys = [
            ('q', 0, 0),
            ('w', 1, 0),
            ('e', 2, 0), // Top row
            ('a', 0, 1),
            ('s', 1, 1),
            ('d', 2, 1), // Middle row
            ('z', 0, 2),
            ('x', 1, 2),
            ('c', 2, 2), // Bottom row
        ];

        for (key, col, row) in area_keys.iter() {
            let x = *col as f64 * area_width;
            let y = *row as f64 * area_height;

            let area = Area {
                key: *key,
                bounds: ScreenBounds {
                    id: 0,
                    x: x as i32,
                    y: y as i32,
                    width: area_width as u32,
                    height: area_height as u32,
                    is_primary: true,
                },
                center: Position {
                    x: (x + area_width / 2.0) as i32,
                    y: (y + area_height / 2.0) as i32,
                    screen_id: None,
                },
                label: format!("{}", key.to_uppercase()),
            };

            self.areas.insert(*key, area);
        }

        debug!(
            "Calculated {} areas for screen {}x{}",
            self.areas.len(),
            self.screen_width,
            self.screen_height
        );
    }

    /// Process input for area mode
    pub fn process_input(
        &mut self,
        input: KeyInput,
        _key_bindings: &KeyBindings,
    ) -> Result<Action> {
        if !self.is_active {
            return Ok(Action::NoAction);
        }

        let key_char = input.key.to_lowercase().next().unwrap_or(input.key);
        let now = std::time::Instant::now();

        // Handle exit keys
        if key_char == ' ' || input.key == '\x1b' {
            // \x1b is escape
            debug!("Exit key pressed in area mode");
            return Ok(Action::DeactivateMode);
        }

        // Valid area keys
        let area_keys = ['q', 'w', 'e', 'a', 's', 'd', 'z', 'x', 'c'];

        if !area_keys.contains(&key_char) {
            debug!("Invalid area key: {}", key_char);
            return Ok(Action::NoAction);
        }

        // Check for timeout (800ms for better UX)
        if let Some(first_time) = self.first_key_time {
            if now.duration_since(first_time).as_millis() > 800 {
                debug!("Area key combination timeout, resetting");
                self.reset_combination_state();
            }
        }

        match self.first_key {
            None => {
                // First key in potential combination - wait briefly for second key
                self.first_key = Some(key_char);
                self.first_key_time = Some(now);
                debug!(
                    "First area key pressed: {}, waiting for potential combination",
                    key_char.to_uppercase()
                );

                // Return action to highlight the first area
                return Ok(Action::HighlightArea(key_char));
            }
            Some(first_key) => {
                // Second key - handle combination
                debug!(
                    "Area key combination: {} + {}",
                    first_key.to_uppercase(),
                    key_char.to_uppercase()
                );

                let combination_position = self.calculate_combination_position(first_key, key_char);
                self.reset_combination_state();

                if let Some(position) = combination_position {
                    debug!("Moving to combination position: {:?}", position);
                    return Ok(Action::MoveCursor(position, AnimationType::Smooth));
                } else {
                    // Invalid combination, move to second key's area
                    if let Some(area) = self.areas.get(&key_char) {
                        let center = area.center.clone();
                        debug!(
                            "Invalid combination, moving to second key area: {}",
                            key_char.to_uppercase()
                        );
                        return Ok(Action::MoveCursor(center, AnimationType::Smooth));
                    }
                }
            }
        }

        Ok(Action::NoAction)
    }

    /// Handle timeout for first key press - move to area if no second key
    pub fn handle_timeout(&mut self) -> Result<Action> {
        if let Some(first_key) = self.first_key {
            if let Some(first_time) = self.first_key_time {
                let now = std::time::Instant::now();
                if now.duration_since(first_time).as_millis() > 400 {
                    // Move to the first key's area after timeout
                    if let Some(area) = self.areas.get(&first_key) {
                        let center = area.center.clone();
                        debug!(
                            "Timeout reached, moving to area {} center: {:?}",
                            first_key.to_uppercase(),
                            center
                        );
                        self.reset_combination_state();
                        return Ok(Action::MoveCursor(center, AnimationType::Smooth));
                    }
                }
            }
        }
        Ok(Action::NoAction)
    }

    /// Calculate position for area key combinations (e.g., Q+E)
    fn calculate_combination_position(
        &self,
        first_key: char,
        second_key: char,
    ) -> Option<Position> {
        let area1 = self.areas.get(&first_key)?;
        let area2 = self.areas.get(&second_key)?;

        // Get area positions in grid (0-2 for each axis)
        let (col1, row1) = self.get_area_grid_position(first_key)?;
        let (col2, row2) = self.get_area_grid_position(second_key)?;

        // Calculate intersection based on area relationships
        let intersection_pos = if col1 == col2 {
            // Same column - vertical intersection
            let y = if row1 < row2 {
                // First area is above second - intersection at bottom of first/top of second
                area1.bounds.y + area1.bounds.height as i32
            } else {
                // First area is below second - intersection at top of first/bottom of second
                area2.bounds.y + area2.bounds.height as i32
            };
            Position {
                x: area1.center.x, // Same column, use x center
                y,
                screen_id: None,
            }
        } else if row1 == row2 {
            // Same row - horizontal intersection
            let x = if col1 < col2 {
                // First area is left of second - intersection at right of first/left of second
                area1.bounds.x + area1.bounds.width as i32
            } else {
                // First area is right of second - intersection at left of first/right of second
                area2.bounds.x + area2.bounds.width as i32
            };
            Position {
                x,
                y: area1.center.y, // Same row, use y center
                screen_id: None,
            }
        } else {
            // Diagonal intersection - use corner point
            let x = if col1 < col2 {
                area1.bounds.x + area1.bounds.width as i32
            } else {
                area1.bounds.x
            };
            let y = if row1 < row2 {
                area1.bounds.y + area1.bounds.height as i32
            } else {
                area1.bounds.y
            };
            Position {
                x,
                y,
                screen_id: None,
            }
        };

        debug!(
            "Combination {} + {} -> grid positions ({},{}) + ({},{}) -> intersection {:?}",
            first_key.to_uppercase(),
            second_key.to_uppercase(),
            col1,
            row1,
            col2,
            row2,
            intersection_pos
        );

        Some(intersection_pos)
    }

    /// Get grid position (column, row) for an area key
    fn get_area_grid_position(&self, key: char) -> Option<(usize, usize)> {
        match key {
            'q' => Some((0, 0)),
            'w' => Some((1, 0)),
            'e' => Some((2, 0)), // Top row
            'a' => Some((0, 1)),
            's' => Some((1, 1)),
            'd' => Some((2, 1)), // Middle row
            'z' => Some((0, 2)),
            'x' => Some((1, 2)),
            'c' => Some((2, 2)), // Bottom row
            _ => None,
        }
    }
}

impl Default for AreaMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input(key: char) -> KeyInput {
        KeyInput {
            key,
            modifiers: vec![],
            timestamp: std::time::SystemTime::now(),
        }
    }

    #[test]
    fn test_area_mode_creation() {
        let mode = AreaMode::new();
        assert!(!mode.is_active());
        assert_eq!(mode.areas.len(), 0);
    }

    #[test]
    fn test_area_mode_activation() {
        let mut mode = AreaMode::new();
        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.activate();

        assert!(mode.is_active());
        assert_eq!(mode.areas.len(), 9);
    }

    #[test]
    fn test_area_calculation() {
        let mut mode = AreaMode::new();
        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.calculate_areas();

        // Test top-left area (Q)
        let q_area = mode.areas.get(&'q').unwrap();
        assert_eq!(q_area.bounds.x, 0);
        assert_eq!(q_area.bounds.y, 0);
        assert_eq!(q_area.bounds.width, 640);
        assert_eq!(q_area.bounds.height, 360);
        assert_eq!(q_area.center.x, 320);
        assert_eq!(q_area.center.y, 180);

        // Test center area (S)
        let s_area = mode.areas.get(&'s').unwrap();
        assert_eq!(s_area.center.x, 960);
        assert_eq!(s_area.center.y, 540);
    }

    #[test]
    fn test_area_input_processing() {
        let mut mode = AreaMode::new();
        let bindings = KeyBindings::default();

        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.activate();

        // Test single key press - should highlight area first
        let action = mode
            .process_input(create_test_input('q'), &bindings)
            .unwrap();
        assert_eq!(action, Action::HighlightArea('q'));

        // Verify that first key state is set
        assert_eq!(mode.first_key, Some('q'));
        assert!(mode.first_key_time.is_some());

        // Manually set the time to simulate timeout
        mode.first_key_time =
            Some(std::time::Instant::now() - std::time::Duration::from_millis(500));

        // Test timeout handling - should move to area after timeout
        let action = mode.handle_timeout().unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.x, 320);
                assert_eq!(pos.y, 180);
            }
            _ => panic!("Expected MoveCursor action after timeout"),
        }

        // State should be reset after timeout
        assert!(mode.first_key.is_none());
        assert!(mode.first_key_time.is_none());
    }

    #[test]
    fn test_area_combination() {
        let mut mode = AreaMode::new();
        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.calculate_areas();

        // Test horizontal combination (same row)
        let pos = mode.calculate_combination_position('q', 'e').unwrap();
        // Q is at (0,0), E is at (2,0) - same row, so intersection at right edge of Q
        assert_eq!(pos.x, 640); // Right edge of Q area
        assert_eq!(pos.y, 180); // Same row y center

        // Test vertical combination (same column)
        let pos = mode.calculate_combination_position('q', 'a').unwrap();
        // Q is at (0,0), A is at (0,1) - same column, so intersection at bottom edge of Q
        assert_eq!(pos.x, 320); // Same column x center
        assert_eq!(pos.y, 360); // Bottom edge of Q area

        // Test diagonal combination
        let pos = mode.calculate_combination_position('q', 'd').unwrap();
        // Q is at (0,0), D is at (2,1) - diagonal, so corner point
        assert_eq!(pos.x, 640); // Right edge of Q
        assert_eq!(pos.y, 360); // Bottom edge of Q
    }

    #[test]
    fn test_grid_position_mapping() {
        let mode = AreaMode::new();

        // Test all area key mappings
        assert_eq!(mode.get_area_grid_position('q'), Some((0, 0)));
        assert_eq!(mode.get_area_grid_position('w'), Some((1, 0)));
        assert_eq!(mode.get_area_grid_position('e'), Some((2, 0)));
        assert_eq!(mode.get_area_grid_position('a'), Some((0, 1)));
        assert_eq!(mode.get_area_grid_position('s'), Some((1, 1)));
        assert_eq!(mode.get_area_grid_position('d'), Some((2, 1)));
        assert_eq!(mode.get_area_grid_position('z'), Some((0, 2)));
        assert_eq!(mode.get_area_grid_position('x'), Some((1, 2)));
        assert_eq!(mode.get_area_grid_position('c'), Some((2, 2)));

        // Test invalid key
        assert_eq!(mode.get_area_grid_position('f'), None);
    }

    #[test]
    fn test_combination_input_processing() {
        let mut mode = AreaMode::new();
        let bindings = KeyBindings::default();

        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.activate();

        // Test first key press - should highlight area
        let action = mode
            .process_input(create_test_input('q'), &bindings)
            .unwrap();
        assert_eq!(action, Action::HighlightArea('q'));

        // Test second key press - should move to combination position
        let action = mode
            .process_input(create_test_input('e'), &bindings)
            .unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, 640); // Right edge of Q area
                assert_eq!(pos.y, 180); // Same row y center
            }
            _ => panic!("Expected MoveCursor action for combination"),
        }
    }

    #[test]
    fn test_timeout_handling() {
        let mut mode = AreaMode::new();
        mode.set_screen_dimensions(1920.0, 1080.0);
        mode.activate();

        // Simulate first key press
        mode.first_key = Some('q');
        mode.first_key_time =
            Some(std::time::Instant::now() - std::time::Duration::from_millis(500));

        // Test timeout handling
        let action = mode.handle_timeout().unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                assert_eq!(pos.x, 320); // Q area center
                assert_eq!(pos.y, 180);
            }
            _ => panic!("Expected MoveCursor action for timeout"),
        }

        // State should be reset
        assert!(mode.first_key.is_none());
        assert!(mode.first_key_time.is_none());
    }

    #[test]
    fn test_exit_keys() {
        let mut mode = AreaMode::new();
        let bindings = KeyBindings::default();

        mode.activate();

        // Test space key exit
        let action = mode
            .process_input(create_test_input(' '), &bindings)
            .unwrap();
        assert_eq!(action, Action::DeactivateMode);

        // Test escape key exit
        let action = mode
            .process_input(create_test_input('\x1b'), &bindings)
            .unwrap();
        assert_eq!(action, Action::DeactivateMode);
    }
}
