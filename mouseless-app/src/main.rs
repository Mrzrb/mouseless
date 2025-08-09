use mouseless_core::{init, AppInfo, Result};
use tauri::{App, AppHandle, Manager};
use tracing::{info, warn};
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
            tauri_commands::show_activation_indicator,
            tauri_commands::hide_activation_indicator,
            tauri_commands::check_accessibility_permissions,
            tauri_commands::request_accessibility_permissions,
            tauri_commands::get_grid_cell_position,
            tauri_commands::test_show_grid,
            tauri_commands::highlight_area,
            tauri_commands::clear_area_highlight
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    
    info!("🚀 Starting Tauri application setup...");
    
    // Initialize UI Manager
    info!("📱 Initializing UI Manager...");
    let ui_manager = UIManager::new(app_handle.clone())?;
    app.manage(Arc::new(Mutex::new(Some(ui_manager))));
    info!("✅ UI Manager initialized and managed");
    
    // Show the main window for testing and control
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show()?;
        main_window.set_title("Mouseless - 网格模式测试")?;
        info!("🪟 Main window shown");
    } else {
        warn!("⚠️ Main window not found");
    }
    
    // Check accessibility permissions on startup
    check_macos_permissions(&app_handle);
    
    info!("🎉 Tauri application setup complete");
    Ok(())
}

fn check_macos_permissions(_app_handle: &AppHandle) {
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