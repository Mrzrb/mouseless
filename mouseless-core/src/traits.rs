use async_trait::async_trait;
use crate::{
    error::{InputResult, MouseResult, UIResult, Result},
    models::*,
};

/// Trait for processing keyboard input events
#[async_trait]
pub trait InputProcessor: Send + Sync {
    /// Process a key event and return the corresponding action
    async fn process_key_event(&self, event: KeyInput) -> InputResult<Action>;
    
    /// Register a global hotkey
    async fn register_hotkey(&mut self, key: char, modifiers: Vec<KeyModifier>) -> InputResult<()>;
    
    /// Update key bindings configuration
    async fn update_bindings(&mut self, bindings: KeyBindings) -> InputResult<()>;
    
    /// Check if the input processor is active
    fn is_active(&self) -> bool;
    
    /// Activate the input processor
    async fn activate(&mut self) -> InputResult<()>;
    
    /// Deactivate the input processor
    async fn deactivate(&mut self) -> InputResult<()>;
}

/// Trait for mouse control operations
pub trait MouseOperations {
    /// Move the mouse cursor to a specific position with animation
    fn move_to(&mut self, position: Position, animation: AnimationType) -> MouseResult<()>;
    
    /// Perform a mouse click
    fn click(&mut self, button: MouseButton) -> MouseResult<()>;
    
    /// Perform mouse scrolling
    fn scroll(&mut self, direction: ScrollDirection, amount: i32) -> MouseResult<()>;
    
    /// Get the current cursor position
    fn get_current_position(&self) -> MouseResult<Position>;
    
    /// Get information about all connected screens
    fn get_screen_bounds(&self) -> MouseResult<Vec<ScreenBounds>>;
    
    /// Set movement speed
    fn set_movement_speed(&mut self, speed: MovementSpeed);
    
    /// Get current movement speed
    fn get_movement_speed(&self) -> MovementSpeed;
}

/// Trait for managing interaction modes
#[async_trait]
pub trait ModeController: Send + Sync {
    /// Activate a specific interaction mode
    async fn activate_mode(&mut self, mode: InteractionMode) -> Result<()>;
    
    /// Deactivate the current mode and return to inactive state
    async fn deactivate_current_mode(&mut self) -> Result<()>;
    
    /// Get the currently active mode
    fn get_current_mode(&self) -> Option<InteractionMode>;
    
    /// Handle input for the current mode
    async fn handle_input(&self, input: KeyInput) -> Result<Action>;
    
    /// Check if any mode is currently active
    fn is_active(&self) -> bool;
    
    /// Get mode history for undo functionality
    fn get_mode_history(&self) -> Vec<InteractionMode>;
}

/// Trait for UI rendering and overlay management
#[async_trait]
pub trait UIRenderer: Send + Sync {
    /// Show grid overlay for grid mode
    async fn show_grid_overlay(&self, grid_config: GridConfig) -> UIResult<()>;
    
    /// Show area overlay for area mode
    async fn show_area_overlay(&self, areas: Vec<Area>) -> UIResult<()>;
    
    /// Show prediction targets for AI mode
    async fn show_prediction_targets(&self, targets: Vec<PredictionTarget>) -> UIResult<()>;
    
    /// Animate cursor movement
    async fn animate_cursor_movement(&self, from: Position, to: Position, animation: AnimationType) -> UIResult<()>;
    
    /// Hide all overlays
    async fn hide_all_overlays(&self) -> UIResult<()>;
    
    /// Show visual feedback for mode activation
    async fn show_mode_indicator(&self, mode: InteractionMode) -> UIResult<()>;
    
    /// Hide mode indicator
    async fn hide_mode_indicator(&self) -> UIResult<()>;
    
    /// Update theme settings
    async fn update_theme(&self, theme: Theme) -> UIResult<()>;
}

/// Trait for prediction models
#[async_trait]
pub trait PredictionModel: Send + Sync {
    /// Predict likely click targets based on screen context
    async fn predict_targets(&self, context: ScreenContext) -> Vec<PredictionTarget>;
    
    /// Update the model with user feedback
    async fn update_model(&mut self, feedback: UserFeedback) -> Result<()>;
    
    /// Get confidence score for a specific target
    fn get_confidence(&self, target: &PredictionTarget) -> f32;
    
    /// Analyze current screen content
    async fn analyze_screen(&self) -> Result<ScreenContext>;
    
    /// Load usage patterns from storage
    async fn load_patterns(&mut self) -> Result<()>;
    
    /// Save usage patterns to storage
    async fn save_patterns(&self) -> Result<()>;
}

/// Key bindings configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyBindings {
    // Movement keys
    pub move_up: char,
    pub move_down: char,
    pub move_left: char,
    pub move_right: char,
    
    // Click keys
    pub left_click: char,
    pub right_click: char,
    pub middle_click: Option<char>,
    
    // Scroll keys
    pub scroll_up: char,
    pub scroll_down: char,
    pub scroll_left: char,
    pub scroll_right: char,
    
    // Mode keys
    pub grid_mode: char,
    pub area_mode: char,
    pub prediction_mode: char,
    
    // Utility keys
    pub speed_toggle: char,
    pub hold_toggle: char,
    pub exit_key: char,
    
    // Screen switching keys
    pub screen_1: char,
    pub screen_2: char,
    pub screen_3: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            // Movement (I/K/J/L)
            move_up: 'i',
            move_down: 'k',
            move_left: 'j',
            move_right: 'l',
            
            // Clicks (N/M)
            left_click: 'n',
            right_click: 'm',
            middle_click: Some(','),
            
            // Scroll (U/O/Y/P)
            scroll_up: 'u',
            scroll_down: 'o',
            scroll_left: 'y',
            scroll_right: 'p',
            
            // Modes
            grid_mode: 'g',
            area_mode: 'a',
            prediction_mode: 'r',
            
            // Utility
            speed_toggle: 'f',
            hold_toggle: 'b',
            exit_key: ' ', // Space key
            
            // Screens
            screen_1: '1',
            screen_2: '2',
            screen_3: '3',
        }
    }
}

/// Theme configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub overlay_opacity: f32,
    pub animation_duration_ms: u64,
    pub glassmorphism_enabled: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            primary_color: "#007AFF".to_string(),
            secondary_color: "#34C759".to_string(),
            background_color: "#000000".to_string(),
            text_color: "#FFFFFF".to_string(),
            overlay_opacity: 0.8,
            animation_duration_ms: 200,
            glassmorphism_enabled: true,
        }
    }
}