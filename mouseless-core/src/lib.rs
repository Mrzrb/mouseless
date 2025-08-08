//! Mouseless Core Library
//! 
//! This library provides the core functionality for the Rust-based keyboard mouse control tool.
//! It includes traits, data models, error handling, and logging utilities that form the foundation
//! of the mouseless application.

pub mod animation;
pub mod error;
pub mod models;
pub mod traits;
pub mod logging;
pub mod input;
pub mod config;
pub mod mouse;
pub mod screen;
pub mod mode;
pub mod basic_mode;
pub mod grid;
pub mod grid_mode;

#[cfg(test)]
pub mod integration_tests;

// Re-export commonly used types
pub use animation::*;
pub use error::{MouselessError, Result};
pub use models::*;
pub use traits::*;
pub use input::*;
pub use config::*;
pub use mouse::*;
pub use screen::*;
pub use mode::*;
pub use basic_mode::*;
pub use grid::*;
pub use grid_mode::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application metadata
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "Mouseless",
            version: VERSION,
            description: "Keyboard-driven mouse control for macOS",
        }
    }
}

/// Initialize the core library
pub fn init() -> Result<()> {
    logging::init_logging()
        .map_err(|e| MouselessError::SystemError(
            std::io::Error::new(std::io::ErrorKind::Other, e)
        ))?;
    
    tracing::info!(
        name = AppInfo::default().name,
        version = AppInfo::default().version,
        "Mouseless core library initialized"
    );
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        let info = AppInfo::default();
        assert_eq!(info.name, "Mouseless");
        assert!(!info.version.is_empty());
        assert!(!info.description.is_empty());
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(100, 200);
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
        assert_eq!(pos.screen_id, None);

        let pos_with_screen = Position::with_screen(100, 200, 1);
        assert_eq!(pos_with_screen.screen_id, Some(1));
    }

    #[test]
    fn test_screen_bounds() {
        let bounds = ScreenBounds {
            id: 1,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        };

        let center = bounds.center();
        assert_eq!(center.x, 960);
        assert_eq!(center.y, 540);
        assert_eq!(center.screen_id, Some(1));

        assert!(bounds.contains(Position::new(500, 500)));
        assert!(!bounds.contains(Position::new(2000, 500)));
    }

    #[test]
    fn test_default_key_bindings() {
        let bindings = KeyBindings::default();
        assert_eq!(bindings.move_up, 'i');
        assert_eq!(bindings.move_down, 'k');
        assert_eq!(bindings.left_click, 'n');
        assert_eq!(bindings.right_click, 'm');
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.overlay_opacity, 0.8);
        assert!(theme.glassmorphism_enabled);
    }
}