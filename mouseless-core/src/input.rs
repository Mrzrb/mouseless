//! Input handling module for global hotkey registration and management
//!
//! This module provides functionality for:
//! - Global hotkey registration using the rdev crate
//! - Configurable activation keys (CapsLock, modifiers)
//! - Double-click detection for activation triggers
//! - Key binding configuration and validation

use async_trait::async_trait;
#[cfg(target_os = "macos")]
use rdev::set_is_main_thread;
use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

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
    /// Convert to rdev Key
    pub fn to_key(&self) -> Key {
        match self {
            ActivationKey::CapsLock => Key::CapsLock,
            ActivationKey::Ctrl => Key::ControlLeft,
            ActivationKey::Shift => Key::ShiftLeft,
            ActivationKey::Command => Key::MetaLeft,
            ActivationKey::Option => Key::Alt,
            ActivationKey::F1 => Key::F1,
            ActivationKey::F2 => Key::F2,
            ActivationKey::F3 => Key::F3,
            ActivationKey::F4 => Key::F4,
            ActivationKey::F5 => Key::F5,
            ActivationKey::F6 => Key::F6,
            ActivationKey::F7 => Key::F7,
            ActivationKey::F8 => Key::F8,
            ActivationKey::F9 => Key::F9,
            ActivationKey::F10 => Key::F10,
            ActivationKey::F11 => Key::F11,
            ActivationKey::F12 => Key::F12,
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
    /// Current key bindings
    key_bindings: Arc<Mutex<KeyBindings>>,
    /// Activation configuration
    activation_config: Arc<Mutex<ActivationConfig>>,
    /// Current activation state
    is_active: Arc<Mutex<bool>>,
    /// Double-click detection state
    double_click_state: Arc<Mutex<DoubleClickState>>,
    /// Registered hotkeys set
    registered_hotkeys: Arc<Mutex<HashSet<Key>>>,
    /// Action sender for processed events
    action_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
    /// Currently pressed modifier keys
    pressed_modifiers: Arc<Mutex<HashSet<Key>>>,
    /// Event loop handle
    event_loop_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> InputResult<Self> {
        Ok(Self {
            key_bindings: Arc::new(Mutex::new(KeyBindings::default())),
            activation_config: Arc::new(Mutex::new(ActivationConfig::default())),
            is_active: Arc::new(Mutex::new(false)),
            double_click_state: Arc::new(Mutex::new(DoubleClickState::default())),
            registered_hotkeys: Arc::new(Mutex::new(HashSet::new())),
            action_sender: Arc::new(Mutex::new(None)),
            pressed_modifiers: Arc::new(Mutex::new(HashSet::new())),
            event_loop_handle: Arc::new(Mutex::new(None)),
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

        // Store the registered hotkey
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(config.trigger_key.to_key());

        // Also register modifier keys if any
        for modifier in &config.modifier_keys {
            let key = match modifier {
                KeyModifier::Shift => Key::ShiftLeft,
                KeyModifier::Ctrl => Key::ControlLeft,
                KeyModifier::Alt => Key::Alt,
                KeyModifier::Cmd => Key::MetaLeft,
            };
            registered.insert(key);
        }

        info!(
            "Registered activation hotkey: {:?} with modifiers: {:?}",
            config.trigger_key, config.modifier_keys
        );

        Ok(())
    }

    /// Start listening for hotkey events
    pub async fn start_event_loop(&self) -> InputResult<()> {
        info!("Starting input event loop with rdev");

        let is_active = Arc::clone(&self.is_active);
        let double_click_state = Arc::clone(&self.double_click_state);
        let activation_config = Arc::clone(&self.activation_config);
        let action_sender = Arc::clone(&self.action_sender);
        let key_bindings = Arc::clone(&self.key_bindings);
        let pressed_modifiers = Arc::clone(&self.pressed_modifiers);
        let registered_hotkeys = Arc::clone(&self.registered_hotkeys);

        let handle = tokio::task::spawn_blocking(move || {
            #[cfg(target_os = "macos")]
            set_is_main_thread(false);
            // Set up a safer event callback that completely avoids macOS threading issues
            if let Err(error) = listen(move |event| {
                // Catch any panics to prevent crashes
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    // Only process specific key events to avoid macOS API issues
                    match event.event_type {
                        rdev::EventType::KeyPress(key) => {
                            info!("Key press detected: {:?}", key);
                            // Only process keys we care about to minimize macOS API calls
                            if Self::is_safe_key(&key) {
                                info!("Processing safe key: {:?}", key);
                                if let Err(e) = Self::process_key_press_safe(
                                    key,
                                    &is_active,
                                    &double_click_state,
                                    &activation_config,
                                    &action_sender,
                                    &key_bindings,
                                    &pressed_modifiers,
                                    &registered_hotkeys,
                                ) {
                                    error!("Error processing key press: {:?}", e);
                                }
                            } else {
                                debug!("Ignoring unsafe key: {:?}", key);
                            }
                        }
                        rdev::EventType::KeyRelease(key) => {
                            // Only track modifier key releases
                            if Self::is_modifier_key(&key) {
                                pressed_modifiers.lock().unwrap().remove(&key);
                            }
                        }
                        _ => {
                            // Completely ignore all other event types
                        }
                    }
                }));

                if let Err(panic_info) = result {
                    error!("Panic caught in event handler: {:?}", panic_info);
                }
            }) {
                error!("Error in rdev listen: {:?}", error);
            }
        });

        *self.event_loop_handle.lock().unwrap() = Some(handle);
        info!("Started input event loop with rdev");
        Ok(())
    }

    /// Process an rdev event synchronously to avoid macOS threading issues
    fn process_rdev_event_sync(
        event: Event,
        is_active: &Arc<Mutex<bool>>,
        double_click_state: &Arc<Mutex<DoubleClickState>>,
        activation_config: &Arc<Mutex<ActivationConfig>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
        key_bindings: &Arc<Mutex<KeyBindings>>,
        pressed_modifiers: &Arc<Mutex<HashSet<Key>>>,
        registered_hotkeys: &Arc<Mutex<HashSet<Key>>>,
    ) -> InputResult<()> {
        match event.event_type {
            rdev::EventType::KeyPress(key) => {
                debug!("Key pressed: {:?}", key);

                // Track modifier keys
                if Self::is_modifier_key(&key) {
                    pressed_modifiers.lock().unwrap().insert(key);
                }

                // Check if this is an activation key
                let config = activation_config.lock().unwrap().clone();
                let registered = registered_hotkeys.lock().unwrap();

                if key == config.trigger_key.to_key() && registered.contains(&key) {
                    // Check if required modifiers are pressed
                    let modifiers_pressed = pressed_modifiers.lock().unwrap();
                    let required_modifiers_pressed = config.modifier_keys.iter().all(|modifier| {
                        let required_key = match modifier {
                            KeyModifier::Shift => Key::ShiftLeft,
                            KeyModifier::Ctrl => Key::ControlLeft,
                            KeyModifier::Alt => Key::Alt,
                            KeyModifier::Cmd => Key::MetaLeft,
                        };
                        modifiers_pressed.contains(&required_key)
                    });

                    if required_modifiers_pressed {
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
                } else if *is_active.lock().unwrap() {
                    // Process key input when mode is active
                    Self::process_active_key_input_sync(key, &key_bindings, &action_sender)?;
                }
            }
            rdev::EventType::KeyRelease(key) => {
                debug!("Key released: {:?}", key);

                // Remove modifier keys from tracking
                if Self::is_modifier_key(&key) {
                    pressed_modifiers.lock().unwrap().remove(&key);
                }
            }
            _ => {
                // Ignore other event types
            }
        }

        Ok(())
    }

    /// Process an rdev event (async version - kept for compatibility)
    async fn process_rdev_event(
        event: Event,
        is_active: &Arc<Mutex<bool>>,
        double_click_state: &Arc<Mutex<DoubleClickState>>,
        activation_config: &Arc<Mutex<ActivationConfig>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
        key_bindings: &Arc<Mutex<KeyBindings>>,
        pressed_modifiers: &Arc<Mutex<HashSet<Key>>>,
        registered_hotkeys: &Arc<Mutex<HashSet<Key>>>,
    ) -> InputResult<()> {
        match event.event_type {
            EventType::KeyPress(key) => {
                debug!("Key pressed: {:?}", key);

                // Track modifier keys
                if Self::is_modifier_key(&key) {
                    pressed_modifiers.lock().unwrap().insert(key);
                }

                // Check if this is an activation key
                let config = activation_config.lock().unwrap().clone();
                let registered = registered_hotkeys.lock().unwrap();

                if key == config.trigger_key.to_key() && registered.contains(&key) {
                    // Check if required modifiers are pressed
                    let modifiers_pressed = pressed_modifiers.lock().unwrap();
                    let required_modifiers_pressed = config.modifier_keys.iter().all(|modifier| {
                        let required_key = match modifier {
                            KeyModifier::Shift => Key::ShiftLeft,
                            KeyModifier::Ctrl => Key::ControlLeft,
                            KeyModifier::Alt => Key::Alt,
                            KeyModifier::Cmd => Key::MetaLeft,
                        };
                        modifiers_pressed.contains(&required_key)
                    });

                    if required_modifiers_pressed {
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
                } else if *is_active.lock().unwrap() {
                    // Process key input when mode is active
                    Self::process_active_key_input(key, &key_bindings, &action_sender).await?;
                }
            }
            EventType::KeyRelease(key) => {
                debug!("Key released: {:?}", key);

                // Remove modifier keys from tracking
                if Self::is_modifier_key(&key) {
                    pressed_modifiers.lock().unwrap().remove(&key);
                }
            }
            _ => {
                // Ignore other event types (mouse events, etc.)
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

    /// Check if a key is a modifier key
    fn is_modifier_key(key: &Key) -> bool {
        matches!(
            key,
            Key::ShiftLeft
                | Key::ShiftRight
                | Key::ControlLeft
                | Key::ControlRight
                | Key::Alt
                | Key::AltGr
                | Key::MetaLeft
                | Key::MetaRight
        )
    }

    /// Check if a key is safe to process (avoids keys that might trigger macOS API calls)
    fn is_safe_key(key: &Key) -> bool {
        matches!(
            key,
            // Alphanumeric keys
            Key::KeyA | Key::KeyB | Key::KeyC | Key::KeyD | Key::KeyE | Key::KeyF |
            Key::KeyG | Key::KeyH | Key::KeyI | Key::KeyJ | Key::KeyK | Key::KeyL |
            Key::KeyM | Key::KeyN | Key::KeyO | Key::KeyP | Key::KeyQ | Key::KeyR |
            Key::KeyS | Key::KeyT | Key::KeyU | Key::KeyV | Key::KeyW | Key::KeyX |
            Key::KeyY | Key::KeyZ |
            Key::Num1 | Key::Num2 | Key::Num3 | Key::Num4 | Key::Num5 |
            Key::Num6 | Key::Num7 | Key::Num8 | Key::Num9 | Key::Num0 |
            // Common punctuation
            Key::Space | Key::Comma | Key::Dot | Key::SemiColon | Key::Quote |
            // Function keys
            Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6 |
            Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12 |
            // Modifier keys
            Key::ShiftLeft | Key::ShiftRight |
            Key::ControlLeft | Key::ControlRight |
            Key::Alt | Key::AltGr |
            Key::MetaLeft | Key::MetaRight |
            // CapsLock (our activation key)
            Key::CapsLock
        )
    }

    /// Process key press safely without triggering macOS API calls
    fn process_key_press_safe(
        key: Key,
        is_active: &Arc<Mutex<bool>>,
        double_click_state: &Arc<Mutex<DoubleClickState>>,
        activation_config: &Arc<Mutex<ActivationConfig>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
        key_bindings: &Arc<Mutex<KeyBindings>>,
        pressed_modifiers: &Arc<Mutex<HashSet<Key>>>,
        registered_hotkeys: &Arc<Mutex<HashSet<Key>>>,
    ) -> InputResult<()> {
        info!("Safe key pressed: {:?}", key);

        // Track modifier keys
        if Self::is_modifier_key(&key) {
            pressed_modifiers.lock().unwrap().insert(key);
            info!("Added modifier key: {:?}", key);
        }

        // Check if this is an activation key
        let config = activation_config.lock().unwrap().clone();
        let registered = registered_hotkeys.lock().unwrap();
        
        info!("Checking activation: key={:?}, trigger_key={:?}, registered={:?}", 
              key, config.trigger_key.to_key(), registered.contains(&key));

        if key == config.trigger_key.to_key() && registered.contains(&key) {
            info!("Activation key detected!");
            // Check if required modifiers are pressed
            let modifiers_pressed = pressed_modifiers.lock().unwrap();
            let required_modifiers_pressed = config.modifier_keys.iter().all(|modifier| {
                let required_key = match modifier {
                    KeyModifier::Shift => Key::ShiftLeft,
                    KeyModifier::Ctrl => Key::ControlLeft,
                    KeyModifier::Alt => Key::Alt,
                    KeyModifier::Cmd => Key::MetaLeft,
                };
                modifiers_pressed.contains(&required_key)
            });

            if required_modifiers_pressed {
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
        } else if *is_active.lock().unwrap() {
            // Process key input when mode is active
            Self::process_active_key_input_sync(key, &key_bindings, &action_sender)?;
        }

        Ok(())
    }

    /// Process key input when mode is active (synchronous version)
    fn process_active_key_input_sync(
        key: Key,
        key_bindings: &Arc<Mutex<KeyBindings>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
    ) -> InputResult<()> {
        let bindings = key_bindings.lock().unwrap().clone();

        // Convert rdev Key to char for comparison with bindings
        let key_char = Self::key_to_char(&key);

        if let Some(ch) = key_char {
            let action = match ch {
                k if k == bindings.move_up => {
                    Action::MoveCursor(Position::new(0, -10), AnimationType::Smooth)
                }
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
                k if k == bindings.prediction_mode => {
                    Action::ActivateMode(InteractionMode::Prediction)
                }
                _ => Action::NoAction,
            };

            if !matches!(action, Action::NoAction) {
                Self::send_action(action, action_sender)?;
                debug!("Processed key '{}' -> action sent", ch);
            }
        }

        Ok(())
    }

    /// Process key input when mode is active (async version - kept for compatibility)
    async fn process_active_key_input(
        key: Key,
        key_bindings: &Arc<Mutex<KeyBindings>>,
        action_sender: &Arc<Mutex<Option<mpsc::UnboundedSender<Action>>>>,
    ) -> InputResult<()> {
        Self::process_active_key_input_sync(key, key_bindings, action_sender)
    }

    /// Convert rdev Key to char (safe version that doesn't trigger macOS APIs)
    fn key_to_char(key: &Key) -> Option<char> {
        // Use a simple mapping that doesn't require macOS keyboard layout APIs
        match key {
            Key::KeyA => Some('a'),
            Key::KeyB => Some('b'),
            Key::KeyC => Some('c'),
            Key::KeyD => Some('d'),
            Key::KeyE => Some('e'),
            Key::KeyF => Some('f'),
            Key::KeyG => Some('g'),
            Key::KeyH => Some('h'),
            Key::KeyI => Some('i'),
            Key::KeyJ => Some('j'),
            Key::KeyK => Some('k'),
            Key::KeyL => Some('l'),
            Key::KeyM => Some('m'),
            Key::KeyN => Some('n'),
            Key::KeyO => Some('o'),
            Key::KeyP => Some('p'),
            Key::KeyQ => Some('q'),
            Key::KeyR => Some('r'),
            Key::KeyS => Some('s'),
            Key::KeyT => Some('t'),
            Key::KeyU => Some('u'),
            Key::KeyV => Some('v'),
            Key::KeyW => Some('w'),
            Key::KeyX => Some('x'),
            Key::KeyY => Some('y'),
            Key::KeyZ => Some('z'),
            Key::Num1 => Some('1'),
            Key::Num2 => Some('2'),
            Key::Num3 => Some('3'),
            Key::Num4 => Some('4'),
            Key::Num5 => Some('5'),
            Key::Num6 => Some('6'),
            Key::Num7 => Some('7'),
            Key::Num8 => Some('8'),
            Key::Num9 => Some('9'),
            Key::Num0 => Some('0'),
            Key::Space => Some(' '),
            Key::Comma => Some(','),
            Key::Dot => Some('.'),
            Key::SemiColon => Some(';'),
            Key::Quote => Some('\''),
            _ => {
                // For any other keys, return None to avoid potential macOS API calls
                debug!("Unsupported key for character conversion: {:?}", key);
                None
            }
        }
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

    /// Stop the event loop
    pub async fn stop_event_loop(&self) -> InputResult<()> {
        if let Some(handle) = self.event_loop_handle.lock().unwrap().take() {
            handle.abort();
            info!("Stopped input event loop");
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

    /// Convert char to rdev Key
    fn char_to_key(ch: char) -> Option<Key> {
        match ch.to_ascii_lowercase() {
            'a' => Some(Key::KeyA),
            'b' => Some(Key::KeyB),
            'c' => Some(Key::KeyC),
            'd' => Some(Key::KeyD),
            'e' => Some(Key::KeyE),
            'f' => Some(Key::KeyF),
            'g' => Some(Key::KeyG),
            'h' => Some(Key::KeyH),
            'i' => Some(Key::KeyI),
            'j' => Some(Key::KeyJ),
            'k' => Some(Key::KeyK),
            'l' => Some(Key::KeyL),
            'm' => Some(Key::KeyM),
            'n' => Some(Key::KeyN),
            'o' => Some(Key::KeyO),
            'p' => Some(Key::KeyP),
            'q' => Some(Key::KeyQ),
            'r' => Some(Key::KeyR),
            's' => Some(Key::KeyS),
            't' => Some(Key::KeyT),
            'u' => Some(Key::KeyU),
            'v' => Some(Key::KeyV),
            'w' => Some(Key::KeyW),
            'x' => Some(Key::KeyX),
            'y' => Some(Key::KeyY),
            'z' => Some(Key::KeyZ),
            '1' => Some(Key::Num1),
            '2' => Some(Key::Num2),
            '3' => Some(Key::Num3),
            '4' => Some(Key::Num4),
            '5' => Some(Key::Num5),
            '6' => Some(Key::Num6),
            '7' => Some(Key::Num7),
            '8' => Some(Key::Num8),
            '9' => Some(Key::Num9),
            '0' => Some(Key::Num0),
            ' ' => Some(Key::Space),
            ',' => Some(Key::Comma),
            '.' => Some(Key::Dot),
            ';' => Some(Key::SemiColon),
            '\'' => Some(Key::Quote),
            _ => None,
        }
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
        // Convert char to rdev Key
        let rdev_key = Self::char_to_key(key).ok_or_else(|| InputError::InvalidKeyBinding {
            binding: format!("Unsupported key: '{}'", key),
        })?;

        // Store the registered hotkey
        let mut registered = self.registered_hotkeys.lock().unwrap();
        registered.insert(rdev_key);

        // Also register modifier keys
        for modifier in &modifiers {
            let modifier_key = match modifier {
                KeyModifier::Shift => Key::ShiftLeft,
                KeyModifier::Ctrl => Key::ControlLeft,
                KeyModifier::Alt => Key::Alt,
                KeyModifier::Cmd => Key::MetaLeft,
            };
            registered.insert(modifier_key);
        }

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

impl Drop for InputHandler {
    fn drop(&mut self) {
        if let Some(handle) = self.event_loop_handle.lock().unwrap().take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_activation_key_to_key() {
        assert_eq!(ActivationKey::CapsLock.to_key(), Key::CapsLock);
        assert_eq!(ActivationKey::Ctrl.to_key(), Key::ControlLeft);
        assert_eq!(ActivationKey::F1.to_key(), Key::F1);
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
        let handler = InputHandler::new().unwrap();
        assert!(!handler.is_active());
    }

    #[tokio::test]
    async fn test_process_key_event() {
        let handler = InputHandler::new().unwrap();

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
    }

    #[tokio::test]
    async fn test_update_bindings() {
        let mut handler = InputHandler::new().unwrap();
        let mut new_bindings = KeyBindings::default();
        new_bindings.move_up = 'w';

        let result = handler.update_bindings(new_bindings.clone()).await;
        assert!(result.is_ok());

        let stored_bindings = handler.key_bindings.lock().unwrap().clone();
        assert_eq!(stored_bindings.move_up, 'w');
    }

    #[test]
    fn test_char_to_key_conversion() {
        assert_eq!(InputHandler::char_to_key('a'), Some(Key::KeyA));
        assert_eq!(InputHandler::char_to_key('z'), Some(Key::KeyZ));
        assert_eq!(InputHandler::char_to_key('1'), Some(Key::Num1));
        assert_eq!(InputHandler::char_to_key(' '), Some(Key::Space));
        assert_eq!(InputHandler::char_to_key('!'), None);
    }

    #[test]
    fn test_key_to_char_conversion() {
        assert_eq!(InputHandler::key_to_char(&Key::KeyA), Some('a'));
        assert_eq!(InputHandler::key_to_char(&Key::KeyZ), Some('z'));
        assert_eq!(InputHandler::key_to_char(&Key::Num1), Some('1'));
        assert_eq!(InputHandler::key_to_char(&Key::Space), Some(' '));
        assert_eq!(InputHandler::key_to_char(&Key::F1), None);
    }

    #[test]
    fn test_is_modifier_key() {
        assert!(InputHandler::is_modifier_key(&Key::ShiftLeft));
        assert!(InputHandler::is_modifier_key(&Key::ControlLeft));
        assert!(InputHandler::is_modifier_key(&Key::Alt));
        assert!(InputHandler::is_modifier_key(&Key::MetaLeft));
        assert!(!InputHandler::is_modifier_key(&Key::KeyA));
        assert!(!InputHandler::is_modifier_key(&Key::Space));
    }
}
