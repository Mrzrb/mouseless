use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

use crate::{
    error::Result,
    models::{Action, AnimationType, InteractionMode, KeyInput, Position},
    traits::KeyBindings,
    GridManager,
};

/// Maximum time between key presses for a valid two-key combination (in milliseconds)
const KEY_COMBINATION_TIMEOUT_MS: u64 = 1000;

/// State for tracking two-key input sequences
#[derive(Debug, Clone)]
struct KeySequenceState {
    first_key: Option<char>,
    first_key_time: Option<SystemTime>,
}

impl Default for KeySequenceState {
    fn default() -> Self {
        Self {
            first_key: None,
            first_key_time: None,
        }
    }
}

/// Grid mode handler for processing two-key input sequences
pub struct GridMode {
    /// Current key sequence state
    key_sequence: KeySequenceState,

    /// Reference to the current grid manager (if any)
    grid_manager: Option<GridManager>,

    /// Whether grid mode is currently active
    is_active: bool,
}

impl GridMode {
    /// Create a new grid mode handler
    pub fn new() -> Self {
        Self {
            key_sequence: KeySequenceState::default(),
            grid_manager: None,
            is_active: false,
        }
    }

    /// Set the grid manager for this mode
    pub fn set_grid_manager(&mut self, grid_manager: Option<GridManager>) {
        self.grid_manager = grid_manager;
        debug!("Grid manager updated in grid mode");
    }

    /// Get the current grid manager
    pub fn get_grid_manager(&self) -> Option<&GridManager> {
        self.grid_manager.as_ref()
    }

    /// Activate grid mode
    pub fn activate(&mut self) {
        self.is_active = true;
        self.reset_key_sequence();
        info!("Grid mode activated");
    }

    /// Deactivate grid mode
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.reset_key_sequence();
        info!("Grid mode deactivated");
    }

    /// Check if grid mode is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Reset the current key sequence
    fn reset_key_sequence(&mut self) {
        self.key_sequence = KeySequenceState::default();
        debug!("Key sequence reset");
    }

    /// Check if the key sequence has timed out
    fn is_sequence_timed_out(&self) -> bool {
        if let Some(first_key_time) = self.key_sequence.first_key_time {
            if let Ok(elapsed) = first_key_time.elapsed() {
                return elapsed > Duration::from_millis(KEY_COMBINATION_TIMEOUT_MS);
            }
        }
        false
    }

    /// Process input for grid mode
    pub fn process_input(
        &mut self,
        input: KeyInput,
        _key_bindings: &KeyBindings,
    ) -> Result<Action> {
        if !self.is_active {
            return Ok(Action::NoAction);
        }

        // Handle exit keys first
        if input.key == ' ' || input.key == '\x1b' {
            // Space or Escape
            return Ok(Action::Exit);
        }

        // Handle mode switching keys only if we're not in the middle of a key sequence
        // and the key is not a valid grid key
        if self.key_sequence.first_key.is_none() {
            match input.key {
                'g' => return Ok(Action::ActivateMode(InteractionMode::Grid)),
                'p' => return Ok(Action::ActivateMode(InteractionMode::Prediction)),
                'b' => return Ok(Action::ActivateMode(InteractionMode::Basic)),
                // 'a' is handled as a grid key since it's a valid first key
                _ => {}
            }
        }

        // Check if we have a grid manager
        if self.grid_manager.is_none() {
            warn!("No grid manager available for grid mode");
            return Ok(Action::NoAction);
        }

        // Check if sequence has timed out
        if self.is_sequence_timed_out() {
            debug!("Key sequence timed out, resetting");
            self.reset_key_sequence();
        }

        // Process the key input
        match self.key_sequence.first_key {
            None => {
                // This is the first key in the sequence
                if self.is_valid_first_key(input.key) {
                    self.key_sequence.first_key = Some(input.key);
                    self.key_sequence.first_key_time = Some(input.timestamp);
                    debug!("First key in sequence: '{}'", input.key);

                    // Return a visual feedback action to show the first key was pressed
                    Ok(Action::NoAction) // For now, just acknowledge the key
                } else {
                    debug!("Invalid first key: '{}'", input.key);
                    Ok(Action::NoAction)
                }
            }
            Some(first_key) => {
                // This is the second key in the sequence
                if self.is_valid_second_key(input.key) {
                    let key_combination = format!("{}{}", first_key, input.key);
                    debug!("Complete key combination: '{}'", key_combination);

                    // Look up the grid cell for this key combination before resetting
                    let result = if let Some(grid_manager) = &self.grid_manager {
                        if let Some(cell) = grid_manager.get_cell_by_keys(&key_combination) {
                            info!(
                                "Grid cell selected: {} -> ({}, {})",
                                key_combination, cell.center_position.x, cell.center_position.y
                            );

                            // Return action to move cursor to the grid cell center
                            Ok(Action::MoveCursor(
                                cell.center_position,
                                AnimationType::Smooth,
                            ))
                        } else {
                            warn!(
                                "No grid cell found for key combination: '{}'",
                                key_combination
                            );
                            Ok(Action::NoAction)
                        }
                    } else {
                        Ok(Action::NoAction)
                    };

                    // Reset sequence for next input
                    self.reset_key_sequence();
                    result
                } else {
                    debug!("Invalid second key: '{}', resetting sequence", input.key);
                    self.reset_key_sequence();
                    Ok(Action::NoAction)
                }
            }
        }
    }

    /// Check if a character is a valid first key in the sequence
    fn is_valid_first_key(&self, key: char) -> bool {
        // First keys are from the home row
        ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'].contains(&key)
    }

    /// Check if a character is a valid second key in the sequence
    fn is_valid_second_key(&self, key: char) -> bool {
        // Second keys are from the top row
        ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].contains(&key)
    }

    /// Get the current key sequence state for visual feedback
    pub fn get_current_sequence(&self) -> Option<String> {
        if let Some(first_key) = self.key_sequence.first_key {
            Some(format!("{}_", first_key))
        } else {
            None
        }
    }

    /// Check if we're waiting for a second key
    pub fn is_waiting_for_second_key(&self) -> bool {
        self.key_sequence.first_key.is_some() && !self.is_sequence_timed_out()
    }

    /// Get all valid key combinations for the current grid
    pub fn get_valid_combinations(&self) -> Vec<String> {
        if let Some(grid_manager) = &self.grid_manager {
            grid_manager
                .get_cells()
                .iter()
                .map(|cell| cell.key_combination.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for GridMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GridConfig, ScreenBounds};
    use std::time::SystemTime;

    fn create_test_input(key: char) -> KeyInput {
        KeyInput {
            key,
            modifiers: vec![],
            timestamp: SystemTime::now(),
        }
    }

    fn create_test_grid_manager() -> GridManager {
        let config = GridConfig {
            rows: 3,
            columns: 3,
            show_labels: true,
            animation_style: AnimationType::Smooth,
            cell_padding: 2,
            border_width: 1,
            opacity: 0.8,
        };

        let screen_bounds = ScreenBounds {
            id: 1,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        };

        GridManager::new(config, screen_bounds).unwrap()
    }

    #[test]
    fn test_grid_mode_creation() {
        let grid_mode = GridMode::new();
        assert!(!grid_mode.is_active());
        assert!(grid_mode.get_grid_manager().is_none());
    }

    #[test]
    fn test_grid_mode_activation() {
        let mut grid_mode = GridMode::new();

        grid_mode.activate();
        assert!(grid_mode.is_active());

        grid_mode.deactivate();
        assert!(!grid_mode.is_active());
    }

    #[test]
    fn test_valid_key_checking() {
        let grid_mode = GridMode::new();

        // Test valid first keys (home row)
        assert!(grid_mode.is_valid_first_key('a'));
        assert!(grid_mode.is_valid_first_key('s'));
        assert!(grid_mode.is_valid_first_key('j'));
        assert!(grid_mode.is_valid_first_key('l'));

        // Test invalid first keys
        assert!(!grid_mode.is_valid_first_key('q'));
        assert!(!grid_mode.is_valid_first_key('1'));
        assert!(!grid_mode.is_valid_first_key('z'));

        // Test valid second keys (top row)
        assert!(grid_mode.is_valid_second_key('q'));
        assert!(grid_mode.is_valid_second_key('w'));
        assert!(grid_mode.is_valid_second_key('i'));
        assert!(grid_mode.is_valid_second_key('p'));

        // Test invalid second keys
        assert!(!grid_mode.is_valid_second_key('a'));
        assert!(!grid_mode.is_valid_second_key('1'));
        assert!(!grid_mode.is_valid_second_key('z'));
    }

    #[test]
    fn test_key_sequence_processing() {
        let mut grid_mode = GridMode::new();
        let grid_manager = create_test_grid_manager();

        grid_mode.set_grid_manager(Some(grid_manager));
        grid_mode.activate();

        let key_bindings = KeyBindings::default();

        // Test first key input
        let action = grid_mode
            .process_input(create_test_input('a'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::NoAction); // Should acknowledge first key
        assert!(grid_mode.is_waiting_for_second_key());

        // Test second key input
        let action = grid_mode
            .process_input(create_test_input('q'), &key_bindings)
            .unwrap();
        match action {
            Action::MoveCursor(pos, AnimationType::Smooth) => {
                // Should move to the center of the 'aq' grid cell
                assert!(pos.x > 0);
                assert!(pos.y > 0);
            }
            _ => panic!("Expected MoveCursor action, got {:?}", action),
        }

        // Sequence should be reset after completion
        assert!(!grid_mode.is_waiting_for_second_key());
    }

    #[test]
    fn test_invalid_key_sequence() {
        let mut grid_mode = GridMode::new();
        let grid_manager = create_test_grid_manager();

        grid_mode.set_grid_manager(Some(grid_manager));
        grid_mode.activate();

        let key_bindings = KeyBindings::default();

        // Test invalid first key
        let action = grid_mode
            .process_input(create_test_input('1'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::NoAction);
        assert!(!grid_mode.is_waiting_for_second_key());

        // Test valid first key followed by invalid second key
        grid_mode
            .process_input(create_test_input('a'), &key_bindings)
            .unwrap();
        assert!(grid_mode.is_waiting_for_second_key());

        let action = grid_mode
            .process_input(create_test_input('1'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::NoAction);
        assert!(!grid_mode.is_waiting_for_second_key()); // Should reset
    }

    #[test]
    fn test_exit_keys() {
        let mut grid_mode = GridMode::new();
        grid_mode.activate();

        let key_bindings = KeyBindings::default();

        // Test space key
        let action = grid_mode
            .process_input(create_test_input(' '), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::Exit);

        // Test escape key
        let action = grid_mode
            .process_input(create_test_input('\x1b'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::Exit);
    }

    #[test]
    fn test_mode_switching_keys() {
        let mut grid_mode = GridMode::new();
        grid_mode.activate();

        let key_bindings = KeyBindings::default();

        // Test switching to prediction mode (p is not a grid key)
        let action = grid_mode
            .process_input(create_test_input('p'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Prediction));

        // Test switching to basic mode (b is not a grid key)
        let action = grid_mode
            .process_input(create_test_input('b'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Basic));

        // Test that 'a' is treated as a grid key, not a mode switch
        let action = grid_mode
            .process_input(create_test_input('a'), &key_bindings)
            .unwrap();
        assert_eq!(action, Action::NoAction); // Should be treated as first key in sequence
    }

    #[test]
    fn test_get_valid_combinations() {
        let mut grid_mode = GridMode::new();
        let grid_manager = create_test_grid_manager();

        // Without grid manager
        assert!(grid_mode.get_valid_combinations().is_empty());

        // With grid manager
        grid_mode.set_grid_manager(Some(grid_manager));
        let combinations = grid_mode.get_valid_combinations();
        assert_eq!(combinations.len(), 9); // 3x3 grid

        // All combinations should be 2 characters
        for combo in combinations {
            assert_eq!(combo.len(), 2);
        }
    }

    #[test]
    fn test_current_sequence_display() {
        let mut grid_mode = GridMode::new();
        let grid_manager = create_test_grid_manager();

        grid_mode.set_grid_manager(Some(grid_manager));
        grid_mode.activate();

        let key_bindings = KeyBindings::default();

        // Initially no sequence
        assert_eq!(grid_mode.get_current_sequence(), None);

        // After first key
        grid_mode
            .process_input(create_test_input('a'), &key_bindings)
            .unwrap();
        assert_eq!(grid_mode.get_current_sequence(), Some("a_".to_string()));

        // After complete sequence
        grid_mode
            .process_input(create_test_input('q'), &key_bindings)
            .unwrap();
        assert_eq!(grid_mode.get_current_sequence(), None);
    }
}
