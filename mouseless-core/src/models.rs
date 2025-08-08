use serde::{Deserialize, Serialize};

/// Represents a position on the screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub screen_id: Option<u32>,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            screen_id: None,
        }
    }
    
    pub fn with_screen(x: i32, y: i32, screen_id: u32) -> Self {
        Self {
            x,
            y,
            screen_id: Some(screen_id),
        }
    }
}

/// Screen boundary information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenBounds {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

impl ScreenBounds {
    pub fn center(&self) -> Position {
        Position::with_screen(
            self.x + (self.width as i32) / 2,
            self.y + (self.height as i32) / 2,
            self.id,
        )
    }
    
    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.x
            && position.x < self.x + self.width as i32
            && position.y >= self.y
            && position.y < self.y + self.height as i32
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Animation types for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationType {
    Instant,
    Linear,
    Smooth,
    Bounce,
}

/// Movement speed settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementSpeed {
    Slow,
    Normal,
    Fast,
}

/// Grid configuration for grid mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub rows: u32,
    pub columns: u32,
    pub show_labels: bool,
    pub animation_style: AnimationType,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            rows: 3,
            columns: 3,
            show_labels: true,
            animation_style: AnimationType::Smooth,
        }
    }
}

/// Area definition for area mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area {
    pub id: u8,
    pub bounds: ScreenBounds,
    pub key: char,
}

/// Prediction target for AI mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionTarget {
    pub position: Position,
    pub confidence: f32,
    pub target_type: TargetType,
    pub shortcut_key: char,
    pub description: Option<String>,
}

/// Types of prediction targets
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Button,
    Link,
    TextField,
    MenuItem,
    Icon,
    Custom(String),
}

/// Screen context for prediction
#[derive(Debug, Clone)]
pub struct ScreenContext {
    pub application_name: String,
    pub window_title: String,
    pub ui_elements: Vec<UIElement>,
    pub timestamp: std::time::SystemTime,
}

/// UI element information
#[derive(Debug, Clone)]
pub struct UIElement {
    pub position: Position,
    pub size: (u32, u32),
    pub element_type: String,
    pub text: Option<String>,
    pub is_clickable: bool,
}

/// User feedback for prediction improvement
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub target_position: Position,
    pub was_correct: bool,
    pub actual_target_type: Option<TargetType>,
    pub context: ScreenContext,
}

/// Key input event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInput {
    pub key: char,
    pub modifiers: Vec<KeyModifier>,
    pub timestamp: std::time::SystemTime,
}

/// Key modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyModifier {
    Shift,
    Ctrl,
    Alt,
    Cmd,
}

/// Actions that can be performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MoveCursor(Position, AnimationType),
    Click(MouseButton),
    Scroll(ScrollDirection, i32),
    ActivateMode(InteractionMode),
    DeactivateMode,
    ToggleSpeed,
    Exit,
    NoAction,
}

/// Interaction modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionMode {
    Basic,
    Grid,
    Area,
    Prediction,
}