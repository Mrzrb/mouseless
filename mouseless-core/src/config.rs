//! Configuration management module
//! 
//! This module provides functionality for:
//! - Loading and saving configuration from JSON/TOML files
//! - Validating configuration values
//! - Managing application settings and key bindings

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    error::{ConfigError, ConfigResult},
    input::ActivationConfig,
    models::{AnimationType, MovementSpeed},
    traits::{KeyBindings, Theme},
};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Activation settings
    pub activation: ActivationConfig,
    /// Movement settings
    pub movement: MovementConfig,
    /// UI settings
    pub ui: UIConfig,
    /// Key bindings
    pub keybindings: KeyBindings,
    /// Theme settings
    pub theme: Theme,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            activation: ActivationConfig::default(),
            movement: MovementConfig::default(),
            ui: UIConfig::default(),
            keybindings: KeyBindings::default(),
            theme: Theme::default(),
        }
    }
}

/// Movement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementConfig {
    /// Default movement speed
    pub default_speed: MovementSpeed,
    /// Fast movement multiplier
    pub fast_speed_multiplier: f32,
    /// Slow movement multiplier
    pub slow_speed_multiplier: f32,
    /// Default animation type
    pub default_animation: AnimationType,
    /// Movement step size in pixels
    pub step_size: i32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            default_speed: MovementSpeed::Normal,
            fast_speed_multiplier: 2.0,
            slow_speed_multiplier: 0.5,
            default_animation: AnimationType::Smooth,
            step_size: 10,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Show visual feedback for mode changes
    pub show_mode_indicators: bool,
    /// Show cursor trail during movement
    pub show_cursor_trail: bool,
    /// Overlay opacity (0.0 to 1.0)
    pub overlay_opacity: f32,
    /// Animation duration in milliseconds
    pub animation_duration_ms: u64,
    /// Enable glassmorphism effects
    pub glassmorphism_enabled: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            show_mode_indicators: true,
            show_cursor_trail: false,
            overlay_opacity: 0.8,
            animation_duration_ms: 200,
            glassmorphism_enabled: true,
        }
    }
}

/// Configuration manager for loading and saving settings
pub struct ConfigManager {
    config_path: PathBuf,
    current_config: AppConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            current_config: AppConfig::default(),
        }
    }

    /// Get the default configuration directory
    pub fn default_config_dir() -> ConfigResult<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::LoadFailed {
                path: "home directory".to_string(),
                reason: "Could not determine home directory".to_string(),
            })?;

        Ok(home_dir.join(".config").join("mouseless"))
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> ConfigResult<PathBuf> {
        Ok(Self::default_config_dir()?.join("config.json"))
    }

    /// Load configuration from file
    pub fn load(&mut self) -> ConfigResult<()> {
        if !self.config_path.exists() {
            info!("Configuration file does not exist, creating default: {:?}", self.config_path);
            self.save()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| ConfigError::LoadFailed {
                path: self.config_path.display().to_string(),
                reason: e.to_string(),
            })?;

        // Try to parse as JSON first, then TOML
        let config = if self.config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| ConfigError::LoadFailed {
                    path: self.config_path.display().to_string(),
                    reason: format!("TOML parsing error: {}", e),
                })?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| ConfigError::LoadFailed {
                    path: self.config_path.display().to_string(),
                    reason: format!("JSON parsing error: {}", e),
                })?
        };

        // Validate the loaded configuration
        self.validate_config(&config)?;
        
        self.current_config = config;
        info!("Loaded configuration from: {:?}", self.config_path);
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> ConfigResult<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::SaveFailed {
                    path: parent.display().to_string(),
                    reason: e.to_string(),
                })?;
        }

        // Serialize based on file extension
        let content = if self.config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(&self.current_config)
                .map_err(|e| ConfigError::SaveFailed {
                    path: self.config_path.display().to_string(),
                    reason: format!("TOML serialization error: {}", e),
                })?
        } else {
            serde_json::to_string_pretty(&self.current_config)
                .map_err(|e| ConfigError::SaveFailed {
                    path: self.config_path.display().to_string(),
                    reason: format!("JSON serialization error: {}", e),
                })?
        };

        fs::write(&self.config_path, content)
            .map_err(|e| ConfigError::SaveFailed {
                path: self.config_path.display().to_string(),
                reason: e.to_string(),
            })?;

        info!("Saved configuration to: {:?}", self.config_path);
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AppConfig {
        &self.current_config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AppConfig) -> ConfigResult<()> {
        self.validate_config(&config)?;
        self.current_config = config;
        Ok(())
    }

    /// Update key bindings
    pub fn update_key_bindings(&mut self, bindings: KeyBindings) -> ConfigResult<()> {
        // Validate key bindings using the input handler validation
        crate::input::InputHandler::validate_key_bindings(&bindings)
            .map_err(|e| ConfigError::ValidationFailed {
                reason: format!("Key binding validation failed: {}", e),
            })?;

        self.current_config.keybindings = bindings;
        Ok(())
    }

    /// Update activation configuration
    pub fn update_activation_config(&mut self, activation: ActivationConfig) -> ConfigResult<()> {
        self.validate_activation_config(&activation)?;
        self.current_config.activation = activation;
        Ok(())
    }

    /// Update movement configuration
    pub fn update_movement_config(&mut self, movement: MovementConfig) -> ConfigResult<()> {
        self.validate_movement_config(&movement)?;
        self.current_config.movement = movement;
        Ok(())
    }

    /// Update UI configuration
    pub fn update_ui_config(&mut self, ui: UIConfig) -> ConfigResult<()> {
        self.validate_ui_config(&ui)?;
        self.current_config.ui = ui;
        Ok(())
    }

    /// Update theme
    pub fn update_theme(&mut self, theme: Theme) -> ConfigResult<()> {
        self.validate_theme(&theme)?;
        self.current_config.theme = theme;
        Ok(())
    }

    /// Validate entire configuration
    fn validate_config(&self, config: &AppConfig) -> ConfigResult<()> {
        // Validate key bindings
        crate::input::InputHandler::validate_key_bindings(&config.keybindings)
            .map_err(|e| ConfigError::ValidationFailed {
                reason: format!("Key binding validation failed: {}", e),
            })?;

        // Validate other components
        self.validate_activation_config(&config.activation)?;
        self.validate_movement_config(&config.movement)?;
        self.validate_ui_config(&config.ui)?;
        self.validate_theme(&config.theme)?;

        Ok(())
    }

    /// Validate activation configuration
    fn validate_activation_config(&self, activation: &ActivationConfig) -> ConfigResult<()> {
        if activation.double_click_timeout_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "activation.double_click_timeout_ms".to_string(),
                value: "0".to_string(),
            });
        }

        if activation.double_click_timeout_ms > 2000 {
            warn!("Double-click timeout is very high: {}ms", activation.double_click_timeout_ms);
        }

        if activation.activation_timeout_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "activation.activation_timeout_ms".to_string(),
                value: "0".to_string(),
            });
        }

        Ok(())
    }

    /// Validate movement configuration
    fn validate_movement_config(&self, movement: &MovementConfig) -> ConfigResult<()> {
        if movement.fast_speed_multiplier <= 0.0 {
            return Err(ConfigError::InvalidValue {
                field: "movement.fast_speed_multiplier".to_string(),
                value: movement.fast_speed_multiplier.to_string(),
            });
        }

        if movement.slow_speed_multiplier <= 0.0 {
            return Err(ConfigError::InvalidValue {
                field: "movement.slow_speed_multiplier".to_string(),
                value: movement.slow_speed_multiplier.to_string(),
            });
        }

        if movement.step_size <= 0 {
            return Err(ConfigError::InvalidValue {
                field: "movement.step_size".to_string(),
                value: movement.step_size.to_string(),
            });
        }

        if movement.step_size > 100 {
            warn!("Movement step size is very large: {}px", movement.step_size);
        }

        Ok(())
    }

    /// Validate UI configuration
    fn validate_ui_config(&self, ui: &UIConfig) -> ConfigResult<()> {
        if ui.overlay_opacity < 0.0 || ui.overlay_opacity > 1.0 {
            return Err(ConfigError::InvalidValue {
                field: "ui.overlay_opacity".to_string(),
                value: ui.overlay_opacity.to_string(),
            });
        }

        if ui.animation_duration_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "ui.animation_duration_ms".to_string(),
                value: "0".to_string(),
            });
        }

        if ui.animation_duration_ms > 2000 {
            warn!("Animation duration is very long: {}ms", ui.animation_duration_ms);
        }

        Ok(())
    }

    /// Validate theme configuration
    fn validate_theme(&self, theme: &Theme) -> ConfigResult<()> {
        // Validate color format (basic hex color validation)
        let colors = [
            (&theme.primary_color, "theme.primary_color"),
            (&theme.secondary_color, "theme.secondary_color"),
            (&theme.background_color, "theme.background_color"),
            (&theme.text_color, "theme.text_color"),
        ];

        for (color, field) in colors {
            if !color.starts_with('#') || color.len() != 7 {
                return Err(ConfigError::InvalidValue {
                    field: field.to_string(),
                    value: color.clone(),
                });
            }

            // Check if the rest are valid hex characters
            if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ConfigError::InvalidValue {
                    field: field.to_string(),
                    value: color.clone(),
                });
            }
        }

        if theme.overlay_opacity < 0.0 || theme.overlay_opacity > 1.0 {
            return Err(ConfigError::InvalidValue {
                field: "theme.overlay_opacity".to_string(),
                value: theme.overlay_opacity.to_string(),
            });
        }

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) {
        self.current_config = AppConfig::default();
        info!("Reset configuration to defaults");
    }

    /// Export configuration to a different file
    pub fn export_to<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let path = path.as_ref();
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::SaveFailed {
                    path: parent.display().to_string(),
                    reason: e.to_string(),
                })?;
        }

        // Serialize based on file extension
        let content = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(&self.current_config)
                .map_err(|e| ConfigError::SaveFailed {
                    path: path.display().to_string(),
                    reason: format!("TOML serialization error: {}", e),
                })?
        } else {
            serde_json::to_string_pretty(&self.current_config)
                .map_err(|e| ConfigError::SaveFailed {
                    path: path.display().to_string(),
                    reason: format!("JSON serialization error: {}", e),
                })?
        };

        fs::write(path, content)
            .map_err(|e| ConfigError::SaveFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        info!("Exported configuration to: {:?}", path);
        Ok(())
    }

    /// Import configuration from a different file
    pub fn import_from<P: AsRef<Path>>(&mut self, path: P) -> ConfigResult<()> {
        let path = path.as_ref();
        
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        // Try to parse as JSON first, then TOML
        let config: AppConfig = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| ConfigError::LoadFailed {
                    path: path.display().to_string(),
                    reason: format!("TOML parsing error: {}", e),
                })?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| ConfigError::LoadFailed {
                    path: path.display().to_string(),
                    reason: format!("JSON parsing error: {}", e),
                })?
        };

        // Validate the imported configuration
        self.validate_config(&config)?;
        
        self.current_config = config;
        info!("Imported configuration from: {:?}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.activation.trigger_key, crate::input::ActivationKey::CapsLock);
        assert_eq!(config.movement.default_speed, MovementSpeed::Normal);
        assert!(config.ui.show_mode_indicators);
        assert_eq!(config.theme.name, "default");
    }

    #[test]
    fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let manager = ConfigManager::new(&config_path);
        assert_eq!(manager.config_path, config_path);
    }

    #[test]
    fn test_save_and_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // Modify config
        manager.current_config.movement.step_size = 20;
        manager.current_config.ui.overlay_opacity = 0.5;
        
        // Save
        assert!(manager.save().is_ok());
        assert!(config_path.exists());
        
        // Load into new manager
        let mut new_manager = ConfigManager::new(&config_path);
        assert!(new_manager.load().is_ok());
        
        assert_eq!(new_manager.current_config.movement.step_size, 20);
        assert_eq!(new_manager.current_config.ui.overlay_opacity, 0.5);
    }

    #[test]
    fn test_save_and_load_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // Modify config
        manager.current_config.movement.step_size = 15;
        manager.current_config.activation.double_click_required = false;
        
        // Save
        assert!(manager.save().is_ok());
        assert!(config_path.exists());
        
        // Load into new manager
        let mut new_manager = ConfigManager::new(&config_path);
        assert!(new_manager.load().is_ok());
        
        assert_eq!(new_manager.current_config.movement.step_size, 15);
        assert!(!new_manager.current_config.activation.double_click_required);
    }

    #[test]
    fn test_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let mut manager = ConfigManager::new(&config_path);
        
        // Test invalid overlay opacity
        let mut invalid_config = AppConfig::default();
        invalid_config.ui.overlay_opacity = 1.5;
        assert!(manager.update_config(invalid_config).is_err());
        
        // Test invalid color format
        let mut invalid_config = AppConfig::default();
        invalid_config.theme.primary_color = "invalid".to_string();
        assert!(manager.update_config(invalid_config).is_err());
        
        // Test invalid step size
        let mut invalid_config = AppConfig::default();
        invalid_config.movement.step_size = -5;
        assert!(manager.update_config(invalid_config).is_err());
    }

    #[test]
    fn test_key_bindings_update() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let mut manager = ConfigManager::new(&config_path);
        
        // Valid key bindings
        let mut valid_bindings = KeyBindings::default();
        valid_bindings.move_up = 'w';
        assert!(manager.update_key_bindings(valid_bindings.clone()).is_ok());
        assert_eq!(manager.current_config.keybindings.move_up, 'w');
        
        // Invalid key bindings (duplicate)
        let mut invalid_bindings = KeyBindings::default();
        invalid_bindings.move_up = 'n'; // Same as left_click
        assert!(manager.update_key_bindings(invalid_bindings).is_err());
    }

    #[test]
    fn test_import_export() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let export_path = temp_dir.path().join("exported.json");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // Modify config
        manager.current_config.movement.step_size = 25;
        
        // Export
        assert!(manager.export_to(&export_path).is_ok());
        assert!(export_path.exists());
        
        // Reset and import
        manager.reset_to_defaults();
        assert_eq!(manager.current_config.movement.step_size, 10); // Default
        
        assert!(manager.import_from(&export_path).is_ok());
        assert_eq!(manager.current_config.movement.step_size, 25);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // Should create default config file
        assert!(manager.load().is_ok());
        assert!(config_path.exists());
    }
}