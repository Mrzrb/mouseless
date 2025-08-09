use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{
    area_mode::AreaMode,
    basic_mode::BasicMode,
    error::Result,
    grid_mode::GridMode,
    models::{Action, InteractionMode, KeyInput},
    traits::{KeyBindings, ModeController},
};

/// Maximum number of modes to keep in history
const MAX_MODE_HISTORY: usize = 10;

/// Event types for mode changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeEvent {
    ModeActivated(InteractionMode),
    ModeDeactivated(InteractionMode),
    ModeChanged {
        from: InteractionMode,
        to: InteractionMode,
    },
    InputProcessed {
        mode: InteractionMode,
        action: Action,
    },
}

/// Mode manager that handles state transitions and mode switching
pub struct ModeManager {
    /// Current active mode (None means inactive)
    current_mode: Arc<Mutex<Option<InteractionMode>>>,

    /// History of previous modes for undo functionality
    mode_history: Arc<Mutex<VecDeque<InteractionMode>>>,

    /// Event broadcaster for mode change notifications
    event_sender: broadcast::Sender<ModeEvent>,

    /// Key bindings configuration
    key_bindings: Arc<Mutex<KeyBindings>>,

    /// Current movement speed multiplier
    movement_speed: Arc<Mutex<f32>>,

    /// Whether speed toggle is in fast mode
    fast_mode: Arc<Mutex<bool>>,

    /// Hold state for click-and-hold functionality
    hold_state: Arc<Mutex<bool>>,

    /// Basic mode implementation
    basic_mode: Arc<Mutex<BasicMode>>,

    /// Grid mode implementation
    grid_mode: Arc<Mutex<GridMode>>,

    /// Area mode implementation
    area_mode: Arc<Mutex<AreaMode>>,
}

impl ModeManager {
    /// Create a new mode manager
    pub fn new(key_bindings: KeyBindings) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            current_mode: Arc::new(Mutex::new(None)),
            mode_history: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_MODE_HISTORY))),
            event_sender,
            key_bindings: Arc::new(Mutex::new(key_bindings)),
            movement_speed: Arc::new(Mutex::new(1.0)),
            fast_mode: Arc::new(Mutex::new(false)),
            hold_state: Arc::new(Mutex::new(false)),
            basic_mode: Arc::new(Mutex::new(BasicMode::new())),
            grid_mode: Arc::new(Mutex::new(GridMode::new())),
            area_mode: Arc::new(Mutex::new(AreaMode::new())),
        }
    }

    /// Subscribe to mode change events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ModeEvent> {
        self.event_sender.subscribe()
    }

    /// Update key bindings
    pub fn update_key_bindings(&self, bindings: KeyBindings) {
        if let Ok(mut kb) = self.key_bindings.lock() {
            *kb = bindings;
            info!("Key bindings updated");
        }
    }

    /// Get current key bindings
    pub fn get_key_bindings(&self) -> KeyBindings {
        self.key_bindings
            .lock()
            .map(|kb| kb.clone())
            .unwrap_or_default()
    }

    /// Add mode to history
    fn add_to_history(&self, mode: InteractionMode) {
        if let Ok(mut history) = self.mode_history.lock() {
            // Remove the mode if it already exists to avoid duplicates
            history.retain(|m| *m != mode);

            // Add to front
            history.push_front(mode);

            // Limit history size
            if history.len() > MAX_MODE_HISTORY {
                history.pop_back();
            }
        }
    }

    /// Send mode event
    fn send_event(&self, event: ModeEvent) {
        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to send mode event: {}", e);
        }
    }

    /// Process basic movement input using BasicMode implementation
    fn process_basic_input(&self, input: KeyInput) -> Result<Action> {
        let bindings = self.get_key_bindings();

        if let Ok(mut basic_mode) = self.basic_mode.lock() {
            let action = basic_mode.process_input(input, &bindings)?;

            // Handle speed toggle action by updating our internal state
            if action == Action::ToggleSpeed {
                self.toggle_speed();
            }

            Ok(action)
        } else {
            warn!("Failed to acquire basic mode lock");
            Ok(Action::NoAction)
        }
    }

    /// Toggle speed mode
    pub fn toggle_speed(&self) {
        if let Ok(mut fast_mode) = self.fast_mode.lock() {
            *fast_mode = !*fast_mode;
            info!(
                "Speed mode toggled to: {}",
                if *fast_mode { "fast" } else { "normal" }
            );

            // Sync with basic mode
            if let Ok(mut basic_mode) = self.basic_mode.lock() {
                basic_mode.toggle_fast_mode();
            }
        }
    }

    /// Check if currently in hold state
    pub fn is_holding(&self) -> bool {
        self.hold_state.lock().map(|state| *state).unwrap_or(false)
    }

    /// Set hold state
    pub fn set_hold_state(&self, holding: bool) {
        if let Ok(mut hold_state) = self.hold_state.lock() {
            *hold_state = holding;

            // Sync with basic mode
            if let Ok(mut basic_mode) = self.basic_mode.lock() {
                basic_mode.set_hold_state(holding);
            }
        }
    }

    /// Set movement speed
    pub fn set_movement_speed(&self, speed: f32) {
        if let Ok(mut movement_speed) = self.movement_speed.lock() {
            *movement_speed = speed;

            // Sync with basic mode
            if let Ok(mut basic_mode) = self.basic_mode.lock() {
                basic_mode.set_movement_speed(speed);
            }
        }
    }

    /// Get current movement speed
    pub fn get_movement_speed(&self) -> f32 {
        self.movement_speed
            .lock()
            .map(|speed| *speed)
            .unwrap_or(1.0)
    }

    /// Set grid manager for grid mode
    pub fn set_grid_manager(&self, grid_manager: Option<crate::GridManager>) {
        if let Ok(mut grid_mode) = self.grid_mode.lock() {
            grid_mode.set_grid_manager(grid_manager);
        }
    }

    /// Get current key sequence for visual feedback (grid mode only)
    pub fn get_current_key_sequence(&self) -> Option<String> {
        if let Ok(grid_mode) = self.grid_mode.lock() {
            grid_mode.get_current_sequence()
        } else {
            None
        }
    }

    /// Check if grid mode is waiting for second key
    pub fn is_waiting_for_second_key(&self) -> bool {
        if let Ok(grid_mode) = self.grid_mode.lock() {
            grid_mode.is_waiting_for_second_key()
        } else {
            false
        }
    }

    /// Set screen dimensions for area mode
    pub fn set_area_screen_dimensions(&self, width: f64, height: f64) {
        if let Ok(mut area_mode) = self.area_mode.lock() {
            area_mode.set_screen_dimensions(width, height);
        }
    }

    /// Get current areas for area mode
    pub fn get_current_areas(&self) -> Vec<crate::models::Area> {
        if let Ok(area_mode) = self.area_mode.lock() {
            area_mode.get_areas()
        } else {
            Vec::new()
        }
    }
}

#[async_trait]
impl ModeController for ModeManager {
    async fn activate_mode(&mut self, mode: InteractionMode) -> Result<()> {
        let previous_mode = {
            let mut current = self.current_mode.lock().unwrap();
            let prev = current.clone();
            *current = Some(mode.clone());
            prev
        };

        // Deactivate previous mode
        if let Some(prev_mode) = &previous_mode {
            match prev_mode {
                InteractionMode::Grid => {
                    if let Ok(mut grid_mode) = self.grid_mode.lock() {
                        grid_mode.deactivate();
                    }
                }
                InteractionMode::Area => {
                    if let Ok(mut area_mode) = self.area_mode.lock() {
                        area_mode.deactivate();
                    }
                }
                _ => {} // Other modes don't need special deactivation yet
            }
        }

        // Activate new mode
        match &mode {
            InteractionMode::Grid => {
                if let Ok(mut grid_mode) = self.grid_mode.lock() {
                    grid_mode.activate();
                }
            }
            InteractionMode::Area => {
                if let Ok(mut area_mode) = self.area_mode.lock() {
                    area_mode.activate();
                }
            }
            _ => {} // Other modes don't need special activation yet
        }

        // Add previous mode to history if it existed
        if let Some(prev) = previous_mode.clone() {
            self.add_to_history(prev.clone());
            self.send_event(ModeEvent::ModeChanged {
                from: prev,
                to: mode.clone(),
            });
        } else {
            self.send_event(ModeEvent::ModeActivated(mode.clone()));
        }

        info!("Activated mode: {:?}", mode);
        Ok(())
    }

    async fn deactivate_current_mode(&mut self) -> Result<()> {
        let current_mode = {
            let mut current = self.current_mode.lock().unwrap();
            let mode = current.take();
            mode
        };

        if let Some(mode) = current_mode {
            // Deactivate the specific mode
            match &mode {
                InteractionMode::Grid => {
                    if let Ok(mut grid_mode) = self.grid_mode.lock() {
                        grid_mode.deactivate();
                    }
                }
                InteractionMode::Area => {
                    if let Ok(mut area_mode) = self.area_mode.lock() {
                        area_mode.deactivate();
                    }
                }
                _ => {} // Other modes don't need special deactivation yet
            }

            self.send_event(ModeEvent::ModeDeactivated(mode.clone()));
            info!("Deactivated mode: {:?}", mode);
        }

        Ok(())
    }

    fn get_current_mode(&self) -> Option<InteractionMode> {
        self.current_mode.lock().unwrap().clone()
    }

    async fn handle_input(&self, input: KeyInput) -> Result<Action> {
        let current_mode = self.get_current_mode();

        let action = match current_mode {
            Some(InteractionMode::Basic) | None => {
                // Basic mode or no mode active - handle basic input
                self.process_basic_input(input.clone())?
            }
            Some(InteractionMode::Grid) => {
                // Grid mode - use grid-specific input handling
                if let Ok(mut grid_mode) = self.grid_mode.lock() {
                    let bindings = self.get_key_bindings();
                    grid_mode.process_input(input.clone(), &bindings)?
                } else {
                    warn!("Failed to acquire grid mode lock, falling back to basic input");
                    self.process_basic_input(input.clone())?
                }
            }
            Some(InteractionMode::Area) => {
                // Area mode - use area-specific input handling
                if let Ok(mut area_mode) = self.area_mode.lock() {
                    let bindings = self.get_key_bindings();
                    area_mode.process_input(input.clone(), &bindings)?
                } else {
                    warn!("Failed to acquire area mode lock, falling back to basic input");
                    self.process_basic_input(input.clone())?
                }
            }
            Some(InteractionMode::Prediction) => {
                // Prediction mode - for now, fall back to basic input
                // TODO: Implement prediction-specific input handling in future tasks
                self.process_basic_input(input.clone())?
            }
        };

        // Send event for processed input
        if let Some(mode) = current_mode {
            self.send_event(ModeEvent::InputProcessed {
                mode,
                action: action.clone(),
            });
        }

        Ok(action)
    }

    fn is_active(&self) -> bool {
        self.current_mode.lock().unwrap().is_some()
    }

    fn get_mode_history(&self) -> Vec<InteractionMode> {
        self.mode_history
            .lock()
            .map(|history| history.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnimationType, MouseButton, Position};
    use std::time::SystemTime;

    fn create_test_input(key: char) -> KeyInput {
        KeyInput {
            key,
            modifiers: vec![],
            timestamp: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn test_mode_activation() {
        let mut manager = ModeManager::new(KeyBindings::default());

        // Initially no mode should be active
        assert!(!manager.is_active());
        assert_eq!(manager.get_current_mode(), None);

        // Activate basic mode
        manager.activate_mode(InteractionMode::Basic).await.unwrap();
        assert!(manager.is_active());
        assert_eq!(manager.get_current_mode(), Some(InteractionMode::Basic));

        // Switch to grid mode
        manager.activate_mode(InteractionMode::Grid).await.unwrap();
        assert_eq!(manager.get_current_mode(), Some(InteractionMode::Grid));

        // Check history
        let history = manager.get_mode_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], InteractionMode::Basic);
    }

    #[tokio::test]
    async fn test_mode_deactivation() {
        let mut manager = ModeManager::new(KeyBindings::default());

        manager.activate_mode(InteractionMode::Basic).await.unwrap();
        assert!(manager.is_active());

        manager.deactivate_current_mode().await.unwrap();
        assert!(!manager.is_active());
        assert_eq!(manager.get_current_mode(), None);
    }

    #[tokio::test]
    async fn test_basic_input_processing() {
        let manager = ModeManager::new(KeyBindings::default());

        // Test movement keys
        let action = manager.handle_input(create_test_input('i')).await.unwrap();
        match action {
            Action::MoveCursor(pos, _) => {
                assert_eq!(pos.x, 0);
                assert!(pos.y < 0); // Moving up
            }
            _ => panic!("Expected MoveCursor action"),
        }

        // Test click keys
        let action = manager.handle_input(create_test_input('n')).await.unwrap();
        assert_eq!(action, Action::Click(MouseButton::Left));

        // Test exit key
        let action = manager.handle_input(create_test_input(' ')).await.unwrap();
        assert_eq!(action, Action::Exit);
    }

    #[tokio::test]
    async fn test_mode_switching_via_input() {
        let manager = ModeManager::new(KeyBindings::default());

        // Test grid mode activation
        let action = manager.handle_input(create_test_input('g')).await.unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Grid));

        // Test area mode activation
        let action = manager.handle_input(create_test_input('a')).await.unwrap();
        assert_eq!(action, Action::ActivateMode(InteractionMode::Area));
    }

    #[test]
    fn test_speed_toggle() {
        let manager = ModeManager::new(KeyBindings::default());

        // Initially should be in normal speed
        assert!(!*manager.fast_mode.lock().unwrap());

        manager.toggle_speed();
        assert!(*manager.fast_mode.lock().unwrap());

        manager.toggle_speed();
        assert!(!*manager.fast_mode.lock().unwrap());
    }

    #[test]
    fn test_hold_state() {
        let manager = ModeManager::new(KeyBindings::default());

        assert!(!manager.is_holding());

        manager.set_hold_state(true);
        assert!(manager.is_holding());

        manager.set_hold_state(false);
        assert!(!manager.is_holding());
    }

    #[test]
    fn test_mode_history_limit() {
        let manager = ModeManager::new(KeyBindings::default());

        // Add more modes than the limit
        for i in 0..15 {
            let mode = match i % 4 {
                0 => InteractionMode::Basic,
                1 => InteractionMode::Grid,
                2 => InteractionMode::Area,
                _ => InteractionMode::Prediction,
            };
            manager.add_to_history(mode);
        }

        let history = manager.get_mode_history();
        assert!(history.len() <= MAX_MODE_HISTORY);
    }

    #[test]
    fn test_event_subscription() {
        let manager = ModeManager::new(KeyBindings::default());
        let mut receiver = manager.subscribe_to_events();

        // Send an event
        manager.send_event(ModeEvent::ModeActivated(InteractionMode::Basic));

        // Should be able to receive it
        let event = receiver.try_recv().unwrap();
        assert_eq!(event, ModeEvent::ModeActivated(InteractionMode::Basic));
    }
}
