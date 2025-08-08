//! Input handling module for global hotkey registration and management
//!
//! This module provides functionality for:
//! - Global hotkey registration using the global-hotkey crate
//! - Configurable activation keys (CapsLock, modifiers)
//! - Double-click detection for activation triggers
//! - Key binding configuration and validation

use async_trait::async_trait;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    error::{InputError, InputResult},
    models::{Action, AnimationType, InteractionMode, KeyInput, KeyModifier, Position},
    traits::{InputProcessor, KeyBindings},
};

/// Configuration for activation behavior
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivationConfig {
    /// Primary activation key
    pub trigger_key: ActivationKey,
    /// Required modifier keys
    pub modifier_keys: Vec<KeyModifier>,
    /// Whether double-click is required for activation
    pub double_click_required: bool,
    /// Timeout for double-click detection in milliseconds
    pub double_click_timeout_ms: u64,
    /// Timeout for activation in milliseconds
    pub activation_timeout_ms: u64,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            trigger_key: ActivationKey::CapsLock,
            modifier_keys: vec![],
            double_click_required: true,
            double_click_timeout_ms: 300,
            activation_timeout_ms: 5000,
        }
    }
}

/// Available activation keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivationKey {
    CapsLock,
    Ctrl,
    Shift,
    Command,
    Option,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl ActivationKey {
    /// Convert to global-hotkey Code
    pub fn to_code(&self) -> Code {
        match self {
            ActivationKey::CapsLock => Code::CapsLock,
            ActivationKey::Ctrl => Code::ControlLeft,
            ActivationKey::Shift => Code::ShiftLeft,
            ActivationKey::Command => Code::MetaLeft,
            ActivationKey::Option => Code::AltLeft,
            ActivationKey::F1 => Code::F1,
            ActivationKey::F2 => Code::F2,
            ActivationKey::F3 => Code::F3,
            ActivationKey::F4 => Code::F4,
            ActivationKey::F5 => Code::F5,
            ActivationKey::F6 => Code::F6,
            ActivationKey::F7 => Code::F7,
            ActivationKey::F8 => Code::F8,
            ActivationKey::F9 => Code::F9,
            ActivationKey::F10 => Code::F10,
            ActivationKey::F11 => Code::F11,
            ActivationKey::F12 => Code::F12,
        }
    }
}

/// State for double-click detection
#[derive(Debug)]
struct DoubleClickState {
    last_press_time: Option<Instant>,
    click_count: u32,
}

impl Default for DoubleClickState {
    fn default() -> Self {
        Self {
            last_press_time: None,
            click_count: 0,
        }
    }
}

/// Input handler for global hotkey management
pub struct InputHandler {
    /// Global hotkey manager
    hotkey_manager: GlobalHotKeyManager,
    /// Event receiver for hotkey events
    event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<GlobalHotKeyEvent>>>>,
    /// Current key bindings
    key_bindings: Arc<Mutex<KeyBindings>>,
    /// Activation configuration
    activation_config: Arc<Mutex<ActivationConfig>>,
    /// Current activation state
    is_active: Arc<Mutex<bool>>,
    /// Double-click detection state
    double_click_state: Arc<Mutex<DoubleClickState>>,
    /// Registered hotkeys map
    registered_hotkeys: Arc<Mutex<HashMap<u32, String>>>,
    /// Action sender for processed events
    action_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> InputResult<Self> {
        let hotkey_manager =
            GlobalHotKeyManager::new().map_err(|e| InputError::EventProcessingFailed {
                reason: format!("Failed to create global hotkey manager: {}", e),
            })?;

        Ok(Self {
            hotkey_manager,
            event_receiver: Arc::new(Mutex::new(None)),
            key_bindings: Arc::new(Mutex::new(KeyBindings::default())),
            activation_config: Arc::new(Mutex::new(ActivationConfig::default())),
            is_active: Arc::new(Mutex::new(false)),
            double_click_state: Arc::new(Mutex::new(DoubleClickState::default())),
            registered_hotkeys: Arc::new(Mutex::new(HashMap::new())),
            action_sender: Arc::new(Mutex::new(None)),
        })
    }

    /// Set up action channel for sending processed actions
    pub fn setup_action_channel(&self) -> mpsc::UnboundedReceiver<Action> {
        let (sender, receiver) = mpsc::unbounded_channel();
        *self.action_sender.lock().unwrap() = Some(sender);
        receiver
    }

    /// Register the activation hotkey
    pub fn register_activation_hotkey(&mut self) -> InputResult<()> {
        let config = self.activation_config.lock().unwrap().clone();

        // Convert modifiers
        let mut modifiers = Modifiers::empty();
        for modifier in &config.modifier_keys {
            match modifier {
                KeyModifier::Shift => modifiers |= Modifiers::SHIFT,
                KeyModifier::Ctrl => modifiers |= Modifiers::CONTROL,
                KeyModifier::Alt => modifiers |= Modifiers::ALT,
                KeyModifier::Cmd => modifiers |= Modifiers::META,
            }
        }

        // Create and register hotkey
        let hotkey = HotKey::new(Some(modifiers), config.trigger_key.to_code());

        self.hotkey_manager.register(hotkey).map_err(|_e| {
            InputError::HotkeyRegistrationFailed {
                key: format!("{:?}", config.trigger_key),
            }
        })?;

        // Store the registered hotkey
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(hotkey.id(), format!("{:?}", config.trigger_key));

        info!(
            "Registered activation hotkey: {:?} with modifiers: {:?}",
            config.trigger_key, config.modifier_keys
        );

        Ok(())
    }

    /// Start listening for hotkey events
    /// Note: This is a simplified implementation. In a real application,
    /// you would need to properly handle the global hotkey event receiver.
    pub async fn start_event_loop(&self) -> InputResult<()> {
        info!("Started input event loop (simplified implementation)");

        // For now, this is a placeholder implementation
        // In a real implementation, you would:
        // 1. Create a proper event receiver from the global-hotkey crate
        // 2. Set up the event processing loop
        // 3. Handle hotkey events and convert them to actions

        Ok(())
    }

    /// Process a hotkey event
    /// Note: This is a simplified implementation for testing purposes
    async fn _process_hotkey_event(
        event: GlobalHotKeyEvent,
        is_active: &Arc<Mutex<bool>>,
        double_click_state: &Arc<Mutex<DoubleClickState>>,
        activation_config: &Arc<Mutex<ActivationConfig>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
        _key_bindings: &Arc<Mutex<KeyBindings>>,
    ) -> InputResult<()> {
        debug!("Processing hotkey event: {:?}", event);

        match event.state {
            HotKeyState::Pressed => {
                let config = activation_config.lock().unwrap().clone();
                let mut active = is_active.lock().unwrap();

                if !*active {
                    // Handle activation logic
                    if config.double_click_required {
                        let should_activate = Self::handle_double_click_detection(
                            &double_click_state,
                            config.double_click_timeout_ms,
                        )?;

                        if should_activate {
                            *active = true;
                            Self::send_action(
                                Action::ActivateMode(InteractionMode::Basic),
                                &action_sender,
                            )?;
                            info!("Activated mouseless mode via double-click");
                        }
                    } else {
                        *active = true;
                        Self::send_action(
                            Action::ActivateMode(InteractionMode::Basic),
                            &action_sender,
                        )?;
                        info!("Activated mouseless mode");
                    }
                }
            }
            HotKeyState::Released => {
                // Handle key release if needed
                debug!("Hotkey released");
            }
        }

        Ok(())
    }

    /// Handle double-click detection logic
    fn handle_double_click_detection(
        double_click_state: &Arc<Mutex<DoubleClickState>>,
        timeout_ms: u64,
    ) -> InputResult<bool> {
        let mut state = double_click_state.lock().unwrap();
        let now = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        let should_activate = if let Some(last_press) = state.last_press_time {
            if now.duration_since(last_press) <= timeout {
                state.click_count += 1;
                state.click_count >= 2
            } else {
                state.click_count = 1;
                false
            }
        } else {
            state.click_count = 1;
            false
        };

        state.last_press_time = Some(now);

        if should_activate {
            // Reset state after successful double-click
            state.click_count = 0;
            state.last_press_time = None;
        }

        Ok(should_activate)
    }

    /// Process key input when mode is active
    async fn process_active_key_input(
        event: GlobalHotKeyEvent,
        _key_bindings: &Arc<Mutex<KeyBindings>>,
        _action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
    ) -> InputResult<()> {
        // This would be expanded to handle actual key processing
        // For now, just log the event
        debug!("Processing active key input: {:?}", event);
        Ok(())
    }

    /// Send an action through the action channel
    fn send_action(
        action: Action,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
    ) -> InputResult<()> {
        let sender_guard = action_sender.lock().unwrap();
        if let Some(sender) = sender_guard.as_ref() {
            sender
                .send(action)
                .map_err(|e| InputError::EventProcessingFailed {
                    reason: format!("Failed to send action: {}", e),
                })?;
        }
        Ok(())
    }

    /// Update activation configuration
    pub fn update_activation_config(&self, config: ActivationConfig) -> InputResult<()> {
        *self.activation_config.lock().unwrap() = config;
        info!("Updated activation configuration");
        Ok(())
    }

    /// Validate key bindings for conflicts
    pub fn validate_key_bindings(bindings: &KeyBindings) -> InputResult<()> {
        let mut used_keys = std::collections::HashSet::new();

        // Collect all keys
        let keys = vec![
            bindings.move_up,
            bindings.move_down,
            bindings.move_left,
            bindings.move_right,
            bindings.left_click,
            bindings.right_click,
            bindings.scroll_up,
            bindings.scroll_down,
            bindings.scroll_left,
            bindings.scroll_right,
            bindings.grid_mode,
            bindings.area_mode,
            bindings.prediction_mode,
            bindings.speed_toggle,
            bindings.hold_toggle,
            bindings.exit_key,
            bindings.screen_1,
            bindings.screen_2,
            bindings.screen_3,
        ];

        // Check for duplicates in main keys first
        for key in keys {
            if !used_keys.insert(key) {
                return Err(InputError::InvalidKeyBinding {
                    binding: format!("Duplicate key binding: '{}'", key),
                });
            }
        }

        // Add optional middle click if present
        if let Some(middle_click) = bindings.middle_click {
            if !used_keys.insert(middle_click) {
                return Err(InputError::InvalidKeyBinding {
                    binding: format!("Duplicate key binding for middle_click: '{}'", middle_click),
                });
            }
        }

        // Validate individual keys - allow alphanumeric, space, and common punctuation
        for key in &used_keys {
            if !key.is_ascii_alphabetic()
                && !key.is_ascii_digit()
                && *key != ' '
                && *key != ','
                && *key != '.'
                && *key != ';'
                && *key != '\''
            {
                return Err(InputError::InvalidKeyBinding {
                    binding: format!("Invalid key character: '{}'", key),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl InputProcessor for InputHandler {
    async fn process_key_event(&self, event: KeyInput) -> InputResult<Action> {
        let bindings = self.key_bindings.lock().unwrap().clone();
        let is_active = *self.is_active.lock().unwrap();

        if !is_active {
            return Ok(Action::NoAction);
        }

        // Map key to action based on bindings
        let action = match event.key {
            k if k == bindings.move_up => Action::MoveCursor(
                Position::new(0, -10), // Relative movement, will be handled by mouse controller
                AnimationType::Smooth,
            ),
            k if k == bindings.move_down => {
                Action::MoveCursor(Position::new(0, 10), AnimationType::Smooth)
            }
            k if k == bindings.move_left => {
                Action::MoveCursor(Position::new(-10, 0), AnimationType::Smooth)
            }
            k if k == bindings.move_right => {
                Action::MoveCursor(Position::new(10, 0), AnimationType::Smooth)
            }
            k if k == bindings.left_click => Action::Click(crate::models::MouseButton::Left),
            k if k == bindings.right_click => Action::Click(crate::models::MouseButton::Right),
            k if k == bindings.exit_key => Action::Exit,
            k if k == bindings.speed_toggle => Action::ToggleSpeed,
            k if k == bindings.grid_mode => Action::ActivateMode(InteractionMode::Grid),
            k if k == bindings.area_mode => Action::ActivateMode(InteractionMode::Area),
            k if k == bindings.prediction_mode => Action::ActivateMode(InteractionMode::Prediction),
            _ => Action::NoAction,
        };

        debug!("Processed key '{}' -> {:?}", event.key, action);
        Ok(action)
    }

    async fn register_hotkey(&mut self, key: char, modifiers: Vec<KeyModifier>) -> InputResult<()> {
        // Convert char to Code (simplified mapping)
        let code = match key.to_ascii_lowercase() {
            'a' => Code::KeyA,
            'b' => Code::KeyB,
            'c' => Code::KeyC,
            'd' => Code::KeyD,
            'e' => Code::KeyE,
            'f' => Code::KeyF,
            'g' => Code::KeyG,
            'h' => Code::KeyH,
            'i' => Code::KeyI,
            'j' => Code::KeyJ,
            'k' => Code::KeyK,
            'l' => Code::KeyL,
            'm' => Code::KeyM,
            'n' => Code::KeyN,
            'o' => Code::KeyO,
            'p' => Code::KeyP,
            'q' => Code::KeyQ,
            'r' => Code::KeyR,
            's' => Code::KeyS,
            't' => Code::KeyT,
            'u' => Code::KeyU,
            'v' => Code::KeyV,
            'w' => Code::KeyW,
            'x' => Code::KeyX,
            'y' => Code::KeyY,
            'z' => Code::KeyZ,
            '1' => Code::Digit1,
            '2' => Code::Digit2,
            '3' => Code::Digit3,
            '4' => Code::Digit4,
            '5' => Code::Digit5,
            '6' => Code::Digit6,
            '7' => Code::Digit7,
            '8' => Code::Digit8,
            '9' => Code::Digit9,
            '0' => Code::Digit0,
            ' ' => Code::Space,
            _ => {
                return Err(InputError::InvalidKeyBinding {
                    binding: format!("Unsupported key: '{}'", key),
                })
            }
        };

        // Convert modifiers
        let mut global_modifiers = Modifiers::empty();
        for modifier in &modifiers {
            match modifier {
                KeyModifier::Shift => global_modifiers |= Modifiers::SHIFT,
                KeyModifier::Ctrl => global_modifiers |= Modifiers::CONTROL,
                KeyModifier::Alt => global_modifiers |= Modifiers::ALT,
                KeyModifier::Cmd => global_modifiers |= Modifiers::META,
            }
        }

        // Create and register hotkey
        let hotkey = HotKey::new(Some(global_modifiers), code);

        self.hotkey_manager.register(hotkey).map_err(|_e| {
            InputError::HotkeyRegistrationFailed {
                key: key.to_string(),
            }
        })?;

        // Store the registered hotkey
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(hotkey.id(), key.to_string());

        info!(
            "Registered hotkey: '{}' with modifiers: {:?}",
            key, modifiers
        );
        Ok(())
    }

    async fn update_bindings(&mut self, bindings: KeyBindings) -> InputResult<()> {
        // Validate bindings first
        Self::validate_key_bindings(&bindings)?;

        // Update stored bindings
        *self.key_bindings.lock().unwrap() = bindings;

        info!("Updated key bindings");
        Ok(())
    }

    fn is_active(&self) -> bool {
        *self.is_active.lock().unwrap()
    }

    async fn activate(&mut self) -> InputResult<()> {
        *self.is_active.lock().unwrap() = true;
        info!("Input handler activated");
        Ok(())
    }

    async fn deactivate(&mut self) -> InputResult<()> {
        *self.is_active.lock().unwrap() = false;
        info!("Input handler deactivated");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_activation_key_to_code() {
        assert_eq!(ActivationKey::CapsLock.to_code(), Code::CapsLock);
        assert_eq!(ActivationKey::Ctrl.to_code(), Code::ControlLeft);
        assert_eq!(ActivationKey::F1.to_code(), Code::F1);
    }

    #[test]
    fn test_default_activation_config() {
        let config = ActivationConfig::default();
        assert_eq!(config.trigger_key, ActivationKey::CapsLock);
        assert!(config.double_click_required);
        assert_eq!(config.double_click_timeout_ms, 300);
    }

    #[test]
    fn test_key_bindings_validation() {
        let valid_bindings = KeyBindings::default();
        let result = InputHandler::validate_key_bindings(&valid_bindings);
        if let Err(e) = &result {
            println!("Validation error: {:?}", e);
            println!("Default bindings: {:?}", valid_bindings);
        }
        assert!(result.is_ok());

        // Test duplicate key binding
        let mut invalid_bindings = KeyBindings::default();
        invalid_bindings.move_up = 'n'; // Same as left_click
        assert!(InputHandler::validate_key_bindings(&invalid_bindings).is_err());
    }

    #[test]
    fn test_double_click_state() {
        let state = DoubleClickState::default();
        assert_eq!(state.click_count, 0);
        assert!(state.last_press_time.is_none());
    }

    #[tokio::test]
    async fn test_input_handler_creation() {
        // Skip this test if we can't create a GlobalHotKeyManager (e.g., in CI)
        if let Ok(handler) = InputHandler::new() {
            assert!(!handler.is_active());
        } else {
            // Test passed - we can't create a GlobalHotKeyManager in this environment
            println!("Skipping test - GlobalHotKeyManager not available");
        }
    }

    #[tokio::test]
    async fn test_process_key_event() {
        // Skip this test if we can't create a GlobalHotKeyManager (e.g., in CI)
        if let Ok(handler) = InputHandler::new() {
            // Test inactive state
            let key_input = KeyInput {
                key: 'i',
                modifiers: vec![],
                timestamp: SystemTime::now(),
            };

            let action = handler.process_key_event(key_input.clone()).await.unwrap();
            assert_eq!(action, Action::NoAction);

            // Activate and test again
            let mut handler = handler;
            handler.activate().await.unwrap();

            let action = handler.process_key_event(key_input).await.unwrap();
            match action {
                Action::MoveCursor(pos, AnimationType::Smooth) => {
                    assert_eq!(pos.x, 0);
                    assert_eq!(pos.y, -10);
                }
                _ => panic!("Expected MoveCursor action"),
            }
        } else {
            // Test passed - we can't create a GlobalHotKeyManager in this environment
            println!("Skipping test - GlobalHotKeyManager not available");
        }
    }

    #[tokio::test]
    async fn test_update_bindings() {
        // Skip this test if we can't create a GlobalHotKeyManager (e.g., in CI)
        if let Ok(mut handler) = InputHandler::new() {
            let mut new_bindings = KeyBindings::default();
            new_bindings.move_up = 'w';

            let result = handler.update_bindings(new_bindings.clone()).await;
            assert!(result.is_ok());

            let stored_bindings = handler.key_bindings.lock().unwrap().clone();
            assert_eq!(stored_bindings.move_up, 'w');
        } else {
            // Test passed - we can't create a GlobalHotKeyManager in this environment
            println!("Skipping test - GlobalHotKeyManager not available");
        }
    }
}
