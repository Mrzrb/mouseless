use crate::{
    error::{MouseError, MouseResult},
    models::{Position, ScreenBounds},
};

/// Screen detection and management utilities
pub struct ScreenManager;

impl ScreenManager {
    /// Detect all connected screens and their bounds
    pub fn detect_screens() -> MouseResult<Vec<ScreenBounds>> {
        #[cfg(target_os = "macos")]
        {
            Self::detect_screens_macos()
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fallback for other platforms
            Self::detect_screens_fallback()
        }
    }

    /// Get the primary screen
    pub fn get_primary_screen() -> MouseResult<ScreenBounds> {
        let screens = Self::detect_screens()?;
        screens.into_iter().find(|s| s.is_primary).ok_or_else(|| {
            MouseError::ScreenDetectionFailed {
                reason: "No primary screen found".to_string(),
            }
        })
    }

    /// Find the screen containing the given position
    pub fn find_screen_for_position(position: Position) -> MouseResult<ScreenBounds> {
        let screens = Self::detect_screens()?;

        if let Some(screen_id) = position.screen_id {
            // Look for specific screen
            screens
                .into_iter()
                .find(|s| s.id == screen_id)
                .ok_or_else(|| MouseError::ScreenDetectionFailed {
                    reason: format!("Screen with ID {} not found", screen_id),
                })
        } else {
            // Find screen containing the position
            let containing_screen = screens.iter().find(|s| s.contains(position)).cloned();
            if let Some(screen) = containing_screen {
                Ok(screen)
            } else {
                // Fall back to primary screen
                screens.into_iter().find(|s| s.is_primary).ok_or_else(|| {
                    MouseError::ScreenDetectionFailed {
                        reason: "No suitable screen found for position".to_string(),
                    }
                })
            }
        }
    }

    /// Get screen by ID
    pub fn get_screen_by_id(screen_id: u32) -> MouseResult<ScreenBounds> {
        let screens = Self::detect_screens()?;
        screens
            .into_iter()
            .find(|s| s.id == screen_id)
            .ok_or_else(|| MouseError::ScreenDetectionFailed {
                reason: format!("Screen with ID {} not found", screen_id),
            })
    }

    #[cfg(target_os = "macos")]
    fn detect_screens_macos() -> MouseResult<Vec<ScreenBounds>> {
        // For now, implement a basic version
        // In a full implementation, this would use CoreGraphics APIs:
        // - CGDisplayBounds() to get screen bounds
        // - CGGetActiveDisplayList() to enumerate displays
        // - CGDisplayIsMain() to identify primary display

        // This is a simplified implementation that assumes common screen setups
        let mut screens = Vec::new();

        // Primary screen (usually the built-in display)
        screens.push(ScreenBounds {
            id: 0,
            x: 0,
            y: 0,
            width: 1920,  // These should be detected from system
            height: 1080, // These should be detected from system
            is_primary: true,
        });

        // Check for common multi-monitor setups
        // This is a placeholder - real implementation would query the system
        if std::env::var("MOUSELESS_MULTI_MONITOR").is_ok() {
            // Add a second monitor to the right
            screens.push(ScreenBounds {
                id: 1,
                x: 1920,
                y: 0,
                width: 1920,
                height: 1080,
                is_primary: false,
            });
        }

        Ok(screens)
    }

    #[cfg(not(target_os = "macos"))]
    fn detect_screens_fallback() -> MouseResult<Vec<ScreenBounds>> {
        // Basic fallback implementation for non-macOS platforms
        Ok(vec![ScreenBounds {
            id: 0,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        }])
    }

    /// Calculate the total desktop bounds across all screens
    pub fn get_desktop_bounds() -> MouseResult<(i32, i32, u32, u32)> {
        let screens = Self::detect_screens()?;

        if screens.is_empty() {
            return Err(MouseError::ScreenDetectionFailed {
                reason: "No screens detected".to_string(),
            });
        }

        let min_x = screens.iter().map(|s| s.x).min().unwrap_or(0);
        let min_y = screens.iter().map(|s| s.y).min().unwrap_or(0);
        let max_x = screens
            .iter()
            .map(|s| s.x + s.width as i32)
            .max()
            .unwrap_or(1920);
        let max_y = screens
            .iter()
            .map(|s| s.y + s.height as i32)
            .max()
            .unwrap_or(1080);

        Ok((min_x, min_y, (max_x - min_x) as u32, (max_y - min_y) as u32))
    }

    /// Map a screen number (1, 2, 3) to screen ID for user convenience
    pub fn map_screen_number_to_id(screen_number: u8) -> MouseResult<u32> {
        let screens = Self::detect_screens()?;

        if screen_number == 0 || screen_number > screens.len() as u8 {
            return Err(MouseError::ScreenDetectionFailed {
                reason: format!("Invalid screen number: {}", screen_number),
            });
        }

        // Screen numbers are 1-based for users, but we need 0-based indexing
        let screen_index = (screen_number - 1) as usize;
        Ok(screens[screen_index].id)
    }

    /// Get the center position of a specific screen
    pub fn get_screen_center(screen_id: u32) -> MouseResult<Position> {
        let screen = Self::get_screen_by_id(screen_id)?;
        Ok(screen.center())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_detection() {
        let screens = ScreenManager::detect_screens();
        assert!(screens.is_ok());

        let screens = screens.unwrap();
        assert!(!screens.is_empty());
        assert!(screens.iter().any(|s| s.is_primary));
    }

    #[test]
    fn test_primary_screen() {
        let primary = ScreenManager::get_primary_screen();
        assert!(primary.is_ok());

        let primary = primary.unwrap();
        assert!(primary.is_primary);
        assert!(primary.width > 0);
        assert!(primary.height > 0);
    }

    #[test]
    fn test_screen_mapping() {
        let screen_id = ScreenManager::map_screen_number_to_id(1);
        assert!(screen_id.is_ok());

        // Invalid screen numbers should fail
        let invalid = ScreenManager::map_screen_number_to_id(0);
        assert!(invalid.is_err());

        let invalid = ScreenManager::map_screen_number_to_id(10);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_desktop_bounds() {
        let bounds = ScreenManager::get_desktop_bounds();
        assert!(bounds.is_ok());

        let (x, y, width, height) = bounds.unwrap();
        assert!(width > 0);
        assert!(height > 0);
    }

    #[test]
    fn test_find_screen_for_position() {
        let pos = Position::new(100, 100);
        let screen = ScreenManager::find_screen_for_position(pos);
        assert!(screen.is_ok());

        let screen = screen.unwrap();
        assert!(screen.contains(pos));
    }

    #[test]
    fn test_screen_center() {
        let screens = ScreenManager::detect_screens().unwrap();
        if let Some(screen) = screens.first() {
            let center = ScreenManager::get_screen_center(screen.id);
            assert!(center.is_ok());

            let center = center.unwrap();
            assert!(screen.contains(center));
        }
    }
}
