use thiserror::Error;

/// Main error type for the mouseless application
#[derive(Debug, Error)]
pub enum MouselessError {
    #[error("Input handling error: {0}")]
    InputError(#[from] InputError),
    
    #[error("Mouse control error: {0}")]
    MouseError(#[from] MouseError),
    
    #[error("UI rendering error: {0}")]
    UIError(#[from] UIError),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("Permission denied: {message}")]
    PermissionError { message: String },
    
    #[error("Prediction model error: {0}")]
    PredictionError(String),
    
    #[error("Mode management error: {0}")]
    ModeError(String),
    
    #[error("System error: {0}")]
    SystemError(#[from] std::io::Error),
}

/// Input-related errors
#[derive(Debug, Error)]
pub enum InputError {
    #[error("Failed to register hotkey: {key}")]
    HotkeyRegistrationFailed { key: String },
    
    #[error("Invalid key binding: {binding}")]
    InvalidKeyBinding { binding: String },
    
    #[error("Key event processing failed: {reason}")]
    EventProcessingFailed { reason: String },
    
    #[error("Input mode not supported: {mode}")]
    UnsupportedInputMode { mode: String },
}

/// Mouse control errors
#[derive(Debug, Error)]
pub enum MouseError {
    #[error("Failed to move cursor to position ({x}, {y}): {reason}")]
    MovementFailed { x: i32, y: i32, reason: String },
    
    #[error("Failed to perform click: {button} - {reason}")]
    ClickFailed { button: String, reason: String },
    
    #[error("Failed to scroll: {direction} - {reason}")]
    ScrollFailed { direction: String, reason: String },
    
    #[error("Screen bounds detection failed: {reason}")]
    ScreenDetectionFailed { reason: String },
    
    #[error("Animation error: {reason}")]
    AnimationError { reason: String },
}

/// UI rendering errors
#[derive(Debug, Error)]
pub enum UIError {
    #[error("Failed to create overlay: {overlay_type}")]
    OverlayCreationFailed { overlay_type: String },
    
    #[error("Failed to render component: {component}")]
    RenderingFailed { component: String },
    
    #[error("Animation failed: {animation_type}")]
    AnimationFailed { animation_type: String },
    
    #[error("Theme loading failed: {theme_name}")]
    ThemeLoadingFailed { theme_name: String },
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration from {path}: {reason}")]
    LoadFailed { path: String, reason: String },
    
    #[error("Failed to save configuration to {path}: {reason}")]
    SaveFailed { path: String, reason: String },
    
    #[error("Invalid configuration value: {field} = {value}")]
    InvalidValue { field: String, value: String },
    
    #[error("Missing required configuration: {field}")]
    MissingField { field: String },
    
    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, MouselessError>;

/// Input-specific result type
pub type InputResult<T> = std::result::Result<T, InputError>;

/// Mouse-specific result type
pub type MouseResult<T> = std::result::Result<T, MouseError>;

/// UI-specific result type
pub type UIResult<T> = std::result::Result<T, UIError>;

/// Config-specific result type
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;