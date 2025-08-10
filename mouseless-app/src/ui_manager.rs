use mouseless_core::{
    GridConfig, GridManager, ModeManager, MouselessError, Position, PredictionTarget, Result,
    ScreenBounds,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tracing::{debug, error, info, warn};

/// UI Manager handles all overlay window creation and management
pub struct UIManager {
    app_handle: AppHandle,
    overlay_windows: HashMap<String, WebviewWindow>,
    current_grid_manager: Option<GridManager>,
    mode_manager: Option<Arc<std::sync::Mutex<ModeManager>>>,
}

impl UIManager {
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        info!("🔧 Initializing UI Manager");

        let ui_manager = Self {
            app_handle,
            overlay_windows: HashMap::new(),
            current_grid_manager: None,
            mode_manager: None,
        };

        info!("✅ UI Manager created successfully");
        Ok(ui_manager)
    }

    /// Set the mode manager for grid mode integration
    pub fn set_mode_manager(&mut self, mode_manager: Arc<std::sync::Mutex<ModeManager>>) {
        self.mode_manager = Some(mode_manager);
    }

    /// Show grid overlay with configurable grid settings
    pub async fn show_grid_overlay(&mut self, grid_config: GridConfig) -> Result<()> {
        info!(
            "🎯 Showing grid overlay with {} rows and {} columns",
            grid_config.rows, grid_config.columns
        );

        // Close existing grid overlay if it exists
        self.hide_overlay("grid").await?;

        //TODO: Get actual screen dimensions from ScreenManager
        //TODO: Support multi-monitor grid overlays
        //TODO: Handle screen resolution changes dynamically
        let screen_bounds = ScreenBounds {
            id: 1,
            x: 0,
            y: 0,
            width: 1920,  //TODO: Get from actual screen detection
            height: 1080, //TODO: Get from actual screen detection
            is_primary: true,
        };

        // Create grid manager with the configuration
        let grid_manager = GridManager::new(grid_config.clone(), screen_bounds.clone())?;

        // Create grid overlay window
        let window = self
            .create_overlay_window(
                "grid",
                "Grid Overlay",
                true,  // transparent
                true,  // always on top
                false, // not resizable
            )
            .await?;

        // Focus the window so it can receive keyboard events
        window.set_focus().map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to focus overlay window: {}", e),
            ))
        })?;

        // Prepare grid data for frontend with animation settings
        let cells = grid_manager.get_cells();
        info!("Grid manager created {} cells", cells.len());

        let grid_data = json!({
            "config": grid_config,
            "cells": cells,
            "screen_bounds": {
                "width": screen_bounds.width,
                "height": screen_bounds.height
            },
            "animation": {
                "type": "appear",
                "duration": 400,
                "easing": "easeOut"
            }
        });

        debug!(
            "Sending grid data: {}",
            serde_json::to_string_pretty(&grid_data).unwrap_or_default()
        );

        // Send grid configuration to the overlay window
        window.emit("configure-grid", &grid_data).map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to configure grid: {}", e),
            ))
        })?;

        // Also emit to all windows to ensure the event is received
        self.app_handle
            .emit("configure-grid", &grid_data)
            .map_err(|e| {
                MouselessError::SystemError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to broadcast grid config: {}", e),
                ))
            })?;

        // Store the grid manager for later use and sync with mode manager
        if let Some(mode_manager) = &self.mode_manager {
            if let Ok(mode_mgr) = mode_manager.lock() {
                mode_mgr.set_grid_manager(Some(grid_manager.clone()));
            }
        }

        self.current_grid_manager = Some(grid_manager);
        self.overlay_windows.insert("grid".to_string(), window);
        Ok(())
    }

    /// Highlight a specific area in the area overlay
    pub async fn highlight_area(&mut self, area_key: char) -> Result<()> {
        info!("🎯 Highlighting area: {}", area_key.to_uppercase());

        if let Some(window) = self.overlay_windows.get("area") {
            let highlight_data = json!({
                "highlightedArea": area_key.to_string().to_lowercase(),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            });

            window
                .emit("highlight-area", &highlight_data)
                .map_err(|e| {
                    MouselessError::SystemError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to highlight area: {}", e),
                    ))
                })?;
        } else {
            warn!("Area overlay not found, cannot highlight area {}", area_key);
        }

        Ok(())
    }

    /// Clear area highlighting
    pub async fn clear_area_highlight(&mut self) -> Result<()> {
        info!("🎯 Clearing area highlight");

        if let Some(window) = self.overlay_windows.get("area") {
            let clear_data = json!({
                "highlightedArea": null,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            });

            window.emit("highlight-area", &clear_data).map_err(|e| {
                MouselessError::SystemError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to clear area highlight: {}", e),
                ))
            })?;
        }

        Ok(())
    }

    /// Show area overlay with 9-region division
    pub async fn show_area_overlay(&mut self) -> Result<()> {
        info!("🎯 Showing area overlay");

        // Close existing area overlay if it exists
        self.hide_overlay("area").await?;

        //TODO: Get actual screen dimensions from ScreenManager
        //TODO: Support multi-monitor area overlays
        //TODO: Handle dynamic screen configuration changes
        let screen_bounds = ScreenBounds {
            id: 0,
            x: 0,
            y: 0,
            width: 1920,  //TODO: Get from actual screen detection
            height: 1080, //TODO: Get from actual screen detection
            is_primary: true,
        };
        let screen_width = screen_bounds.width as f64;
        let screen_height = screen_bounds.height as f64;

        info!("📐 Screen dimensions: {}x{}", screen_width, screen_height);

        // Calculate 9 areas
        let area_width = screen_width / 3.0;
        let area_height = screen_height / 3.0;

        let area_keys = [
            ('q', 0, 0),
            ('w', 1, 0),
            ('e', 2, 0), // Top row
            ('a', 0, 1),
            ('s', 1, 1),
            ('d', 2, 1), // Middle row
            ('z', 0, 2),
            ('x', 1, 2),
            ('c', 2, 2), // Bottom row
        ];

        let mut areas = Vec::new();
        for (key, col, row) in area_keys.iter() {
            let x = *col as f64 * area_width;
            let y = *row as f64 * area_height;

            areas.push(json!({
                "key": key.to_string(),
                "bounds": {
                    "x": x,
                    "y": y,
                    "width": area_width,
                    "height": area_height
                },
                "center": {
                    "x": x + area_width / 2.0,
                    "y": y + area_height / 2.0
                },
                "label": key.to_uppercase().to_string()
            }));
        }

        let area_data = json!({
            "areas": areas,
            "screen_bounds": {
                "width": screen_width,
                "height": screen_height
            }
        });

        info!("📋 Area data created with {} areas", areas.len());

        // Create area overlay window
        let window = self
            .create_overlay_window(
                "area",
                "Area Overlay",
                true,  // transparent
                true,  // always on top
                false, // not resizable
            )
            .await?;

        // Send area configuration to the frontend
        window.emit("configure-area", &area_data).map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to configure area: {}", e),
            ))
        })?;

        info!("✅ Area overlay configured and displayed");
        self.overlay_windows.insert("area".to_string(), window);
        Ok(())
    }

    /// Show prediction targets with confidence indicators
    pub async fn show_prediction_targets(&mut self, targets: Vec<PredictionTarget>) -> Result<()> {
        info!("Showing {} prediction targets", targets.len());

        // Close existing prediction overlay if it exists
        self.hide_overlay("prediction").await?;

        // Create prediction overlay window
        let window = self
            .create_overlay_window(
                "prediction",
                "Prediction Overlay",
                true,  // transparent
                true,  // always on top
                false, // not resizable
            )
            .await?;

        // Send prediction targets to the frontend
        window.emit("configure-prediction", &targets).map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to configure prediction: {}", e),
            ))
        })?;

        self.overlay_windows
            .insert("prediction".to_string(), window);
        Ok(())
    }

    /// Hide all overlay windows
    pub async fn hide_all_overlays(&mut self) -> Result<()> {
        info!("Hiding all overlays");

        let overlay_names: Vec<String> = self.overlay_windows.keys().cloned().collect();

        for overlay_name in overlay_names {
            self.hide_overlay(&overlay_name).await?;
        }

        Ok(())
    }

    /// Hide a specific overlay by name
    pub async fn hide_overlay(&mut self, overlay_name: &str) -> Result<()> {
        if let Some(window) = self.overlay_windows.get(overlay_name) {
            debug!("Hiding overlay with animation: {}", overlay_name);

            // Send disappear animation command for grid overlay
            if overlay_name == "grid" {
                let animation_data = json!({
                    "animation": {
                        "type": "disappear",
                        "duration": 300,
                        "easing": "easeIn"
                    }
                });

                if let Err(e) = window.emit("animate-grid-disappear", &animation_data) {
                    warn!("Failed to send disappear animation: {}", e);
                }

                // Wait a bit for animation to complete before closing
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            }

            // Remove from our tracking and close the window
            if let Some(window) = self.overlay_windows.remove(overlay_name) {
                window.close().map_err(|e| {
                    MouselessError::SystemError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to close overlay {}: {}", overlay_name, e),
                    ))
                })?;
            }
        }
        Ok(())
    }

    /// Create a new overlay window with specified properties
    async fn create_overlay_window(
        &self,
        label: &str,
        title: &str,
        transparent: bool,
        always_on_top: bool,
        resizable: bool,
    ) -> Result<WebviewWindow> {
        debug!("Creating overlay window: {}", label);

        let mut builder = WebviewWindowBuilder::new(
            &self.app_handle,
            label,
            WebviewUrl::App("overlay.html".into()),
        )
        .title(title)
        .inner_size(1920.0, 1080.0) // Full screen size, will be adjusted based on actual screen
        .position(0.0, 0.0)
        .decorations(false)
        .transparent(transparent)
        .always_on_top(always_on_top)
        .resizable(resizable)
        .skip_taskbar(false) // Allow in taskbar so it can receive focus
        .visible(false) // Start hidden, show after configuration
        .focused(true); // Request focus when shown

        // macOS specific settings for overlay windows
        #[cfg(target_os = "macos")]
        {
            //TODO: Add more macOS-specific window settings for better overlay behavior
            //TODO: Handle macOS window level settings for proper overlay positioning
            //TODO: Implement proper focus management for macOS overlays
            builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        let window = builder.build().map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create overlay window: {}", e),
            ))
        })?;

        // Show the window after creation
        window.show().map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to show overlay window: {}", e),
            ))
        })?;

        // Wait a bit for the window to fully load
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(window)
    }

    /// Get the current screen dimensions for overlay sizing
    pub fn get_screen_dimensions(&self) -> Result<(f64, f64)> {
        //TODO: Integrate with ScreenManager to get actual screen dimensions
        //TODO: Support multi-monitor setups with proper screen detection
        //TODO: Handle screen resolution changes dynamically
        Ok((1920.0, 1080.0))
    }

    /// Animate cursor movement with smooth transitions
    pub async fn animate_cursor_movement(&self, from: Position, to: Position) -> Result<()> {
        info!(
            "Animating cursor movement from ({}, {}) to ({}, {})",
            from.x, from.y, to.x, to.y
        );

        //TODO: Implement visual cursor movement animation
        //TODO: Show cursor trail or path visualization
        //TODO: Integrate with mouse controller for smooth movement
        //TODO: Add configurable animation settings (speed, easing)
        //TODO: Handle animation cancellation and interruption

        Ok(())
    }

    /// Show system-wide visual indicator for active state
    pub async fn show_activation_indicator(&mut self) -> Result<()> {
        info!("Showing activation indicator");

        // Close existing activation indicator if it exists
        self.hide_overlay("activation-indicator").await?;

        // Create a subtle activation indicator overlay
        let window = self
            .create_overlay_window(
                "activation-indicator",
                "Activation Indicator",
                true,  // transparent
                true,  // always on top
                false, // not resizable
            )
            .await?;

        // Send activation indicator configuration to the frontend
        window
            .emit("show-activation-indicator", &serde_json::json!({}))
            .map_err(|e| {
                MouselessError::SystemError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to show activation indicator: {}", e),
                ))
            })?;

        self.overlay_windows
            .insert("activation-indicator".to_string(), window);
        Ok(())
    }

    /// Hide system-wide visual indicator
    pub async fn hide_activation_indicator(&mut self) -> Result<()> {
        info!("Hiding activation indicator");
        self.hide_overlay("activation-indicator").await
    }

    /// Get the current grid manager
    pub fn get_grid_manager(&self) -> Option<&GridManager> {
        self.current_grid_manager.as_ref()
    }

    /// Get grid cell position by key combination
    pub fn get_grid_cell_position(&self, key_combination: &str) -> Option<Position> {
        self.current_grid_manager
            .as_ref()
            .and_then(|manager| manager.get_cell_by_keys(key_combination))
            .map(|cell| cell.center_position)
    }

    /// Show visual feedback for key sequence input
    pub async fn show_key_sequence_feedback(&mut self, sequence: String) -> Result<()> {
        info!("Showing key sequence feedback: {}", sequence);

        // Close existing feedback overlay if it exists
        self.hide_overlay("key-feedback").await?;

        // Create key feedback overlay window
        let window = self
            .create_overlay_window(
                "key-feedback",
                "Key Sequence Feedback",
                true,  // transparent
                true,  // always on top
                false, // not resizable
            )
            .await?;

        // Send key sequence data to the frontend
        let feedback_data = json!({
            "sequence": sequence,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        });

        window
            .emit("show-key-feedback", &feedback_data)
            .map_err(|e| {
                MouselessError::SystemError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to show key feedback: {}", e),
                ))
            })?;

        self.overlay_windows
            .insert("key-feedback".to_string(), window);
        Ok(())
    }

    /// Hide key sequence feedback
    pub async fn hide_key_sequence_feedback(&mut self) -> Result<()> {
        info!("Hiding key sequence feedback");
        self.hide_overlay("key-feedback").await
    }

    /// Move mouse to specific position
    pub async fn move_mouse_to_position(&mut self, x: i32, y: i32) -> Result<()> {
        use mouseless_core::{AnimationType, MouseController, MouseOperations};

        info!("🖱️ Moving mouse to position ({}, {})", x, y);

        // Create mouse controller on demand to avoid Send/Sync issues
        let mut controller = MouseController::new().map_err(|e| {
            MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create mouse controller: {}", e),
            ))
        })?;

        let position = Position::new(x, y);
        controller
            .move_to(position, AnimationType::Smooth)
            .map_err(|e| {
                MouselessError::SystemError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to move mouse: {}", e),
                ))
            })?;

        info!("✅ Mouse moved successfully to ({}, {})", x, y);
        Ok(())
    }

    /// Move mouse to grid cell by key combination
    pub async fn move_mouse_to_grid_cell(&mut self, key_combination: &str) -> Result<()> {
        info!("🎯 Moving mouse to grid cell: {}", key_combination);

        if let Some(position) = self.get_grid_cell_position(key_combination) {
            self.move_mouse_to_position(position.x, position.y).await?;

            // Hide grid overlay after successful movement
            self.hide_all_overlays().await?;

            info!(
                "✅ Mouse moved to grid cell {} at ({}, {})",
                key_combination, position.x, position.y
            );
        } else {
            warn!(
                "⚠️ Grid cell not found for key combination: {}",
                key_combination
            );
            return Err(MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Grid cell not found for key combination: {}",
                    key_combination
                ),
            )));
        }

        Ok(())
    }
}

impl Drop for UIManager {
    fn drop(&mut self) {
        // Clean up all overlay windows when the UI manager is dropped
        let overlay_names: Vec<String> = self.overlay_windows.keys().cloned().collect();

        for overlay_name in overlay_names {
            if let Some(window) = self.overlay_windows.remove(&overlay_name) {
                if let Err(e) = window.close() {
                    error!(
                        "Failed to close overlay {} during cleanup: {}",
                        overlay_name, e
                    );
                }
            }
        }
    }
}
