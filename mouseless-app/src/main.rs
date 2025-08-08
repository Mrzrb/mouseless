use mouseless_core::{init, AppInfo, Result};
use tauri::{App, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod ui_manager;
mod tauri_commands;

use ui_manager::UIManager;

// Global state for the UI manager
type UIManagerState = Arc<Mutex<Option<UIManager>>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the core library first
    if let Err(e) = init() {
        eprintln!("Failed to initialize core library: {}", e);
        return;
    }

    let app_info = AppInfo::default();
    info!(
        "Starting {} v{}",
        app_info.name,
        app_info.version
    );

    tauri::Builder::default()
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            tauri_commands::show_grid_overlay,
            tauri_commands::show_area_overlay,
            tauri_commands::show_prediction_targets,
            tauri_commands::hide_all_overlays,
            tauri_commands::check_accessibility_permissions,
            tauri_commands::request_accessibility_permissions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    
    // Initialize UI Manager
    let ui_manager = UIManager::new(app_handle.clone())?;
    app.manage(Arc::new(Mutex::new(Some(ui_manager))));
    
    // Hide the main window initially (we'll use overlay windows)
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.hide()?;
    }
    
    // Check accessibility permissions on startup
    check_macos_permissions(&app_handle);
    
    info!("Tauri application setup complete");
    Ok(())
}

fn check_macos_permissions(app_handle: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Check if accessibility permissions are granted
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of every process")
            .output();
            
        match output {
            Ok(_) => {
                info!("Accessibility permissions are granted");
            }
            Err(_) => {
                warn!("Accessibility permissions may not be granted");
                // We'll handle this in the UI with user guidance
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_initialization() {
        // Test that the core library can be initialized
        let result = init();
        assert!(result.is_ok());
    }
}