use mouseless_core::{MouselessError, Result, Position, GridConfig, PredictionTarget, AnimationStyle};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WebviewWindow};
use tracing::{debug, error, info, warn};
use std::collections::HashMap;
use serde_json::json;

/// UI Manager handles all overlay window creation and management
pub struct UIManager {
    app_handle: AppHandle,
    overlay_windows: HashMap<String, WebviewWindow>,
}

impl UIManager {
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        info!("Initializing UI Manager");
        
        Ok(Self {
            app_handle,
            overlay_windows: HashMap::new(),
        })
    }
    
    /// Show grid overlay with configurable grid settings
    pub async fn show_grid_overlay(&mut self, grid_config: GridConfig) -> Result<()> {
        info!("Showing grid overlay with {} rows and {} columns", 
              grid_config.rows, grid_config.columns);
        
        // Close existing grid overlay if it exists
        self.hide_overlay("grid").await?;
        
        // Create grid overlay window
        let window = self.create_overlay_window(
            "grid",
            "Grid Overlay",
            true, // transparent
            true, // always on top
            false, // not resizable
        ).await?;
        
        // Send grid configuration to the frontend
        window.emit("configure-grid", &grid_config)
            .map_err(|e| MouselessError::SystemError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to configure grid: {}", e))
            ))?;
        
        self.overlay_windows.insert("grid".to_string(), window);
        Ok(())
    }
    
    /// Show area overlay with 9-region division
    pub async fn show_area_overlay(&mut self) -> Result<()> {
        info!("Showing area overlay");
        
        // Close existing area overlay if it exists
        self.hide_overlay("area").await?;
        
        // Create area overlay window
        let window = self.create_overlay_window(
            "area",
            "Area Overlay",
            true, // transparent
            true, // always on top
            false, // not resizable
        ).await?;
        
        // Send area configuration to the frontend
        window.emit("configure-area", &json!({}))
            .map_err(|e| MouselessError::SystemError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to configure area: {}", e))
            ))?;
        
        self.overlay_windows.insert("area".to_string(), window);
        Ok(())
    }
    
    /// Show prediction targets with confidence indicators
    pub async fn show_prediction_targets(&mut self, targets: Vec<PredictionTarget>) -> Result<()> {
        info!("Showing {} prediction targets", targets.len());
        
        // Close existing prediction overlay if it exists
        self.hide_overlay("prediction").await?;
        
        // Create prediction overlay window
        let window = self.create_overlay_window(
            "prediction",
            "Prediction Overlay",
            true, // transparent
            true, // always on top
            false, // not resizable
        ).await?;
        
        // Send prediction targets to the frontend
        window.emit("configure-prediction", &targets)
            .map_err(|e| MouselessError::SystemError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to configure prediction: {}", e))
            ))?;
        
        self.overlay_windows.insert("prediction".to_string(), window);
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
        if let Some(window) = self.overlay_windows.remove(overlay_name) {
            debug!("Hiding overlay: {}", overlay_name);
            window.close()
                .map_err(|e| MouselessError::SystemError(
                    std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to close overlay {}: {}", overlay_name, e))
                ))?;
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
            WebviewUrl::App("overlay.html".into())
        )
        .title(title)
        .inner_size(1920.0, 1080.0) // Full screen size, will be adjusted based on actual screen
        .position(0.0, 0.0)
        .decorations(false)
        .transparent(transparent)
        .always_on_top(always_on_top)
        .resizable(resizable)
        .skip_taskbar(true)
        .visible(false); // Start hidden, show after configuration
        
        // macOS specific settings for overlay windows
        #[cfg(target_os = "macos")]
        {
            builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
        }
        
        let window = builder.build()
            .map_err(|e| MouselessError::SystemError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create overlay window: {}", e))
            ))?;
        
        // Show the window after creation
        window.show()
            .map_err(|e| MouselessError::SystemError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to show overlay window: {}", e))
            ))?;
        
        Ok(window)
    }
    
    /// Get the current screen dimensions for overlay sizing
    pub fn get_screen_dimensions(&self) -> Result<(f64, f64)> {
        // For now, return a default size. This will be enhanced with actual screen detection
        // in future tasks when we integrate with the screen module
        Ok((1920.0, 1080.0))
    }
}

impl Drop for UIManager {
    fn drop(&mut self) {
        // Clean up all overlay windows when the UI manager is dropped
        let overlay_names: Vec<String> = self.overlay_windows.keys().cloned().collect();
        
        for overlay_name in overlay_names {
            if let Some(window) = self.overlay_windows.remove(&overlay_name) {
                if let Err(e) = window.close() {
                    error!("Failed to close overlay {} during cleanup: {}", overlay_name, e);
                }
            }
        }
    }
}