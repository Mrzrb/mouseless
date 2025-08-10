use mouseless_core::{init, AppInfo, MouseService, Result};
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};
use tracing::{info, warn};

mod tauri_commands;
mod ui_manager;

use ui_manager::UIManager;

// Global state types for Tauri state management

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the core library first
    if let Err(e) = init() {
        eprintln!("Failed to initialize core library: {}", e);
        return;
    }

    let app_info = AppInfo::default();
    info!("Starting {} v{}", app_info.name, app_info.version);

    tauri::Builder::default()
        .setup(setup_app)
        .on_menu_event(handle_menu_event)
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
            tauri_commands::clear_area_highlight,
            tauri_commands::move_mouse_to_position,
            tauri_commands::move_mouse_to_grid_cell,
            tauri_commands::load_settings,
            tauri_commands::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    info!("🚀 Starting Tauri application setup...");
    
    // Hide from dock on macOS
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
        unsafe {
            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory);
        }
    }

    //TODO: Initialize ConfigManager and load ~/.mouseless.toml
    //TODO: Set up SIGHUP signal handler for configuration reload
    //TODO: Initialize InputHandler with global hotkey registration
    //TODO: Initialize ModeManager with configuration-driven settings
    //TODO: Set up inter-component communication channels

    // Initialize UI Manager
    info!("📱 Initializing UI Manager...");
    let ui_manager = UIManager::new(app_handle.clone())?;
    app.manage(Arc::new(Mutex::new(Some(ui_manager))));
    info!("✅ UI Manager initialized and managed");

    // Initialize Mouse Service
    info!("🖱️ Initializing Mouse Service...");
    let mouse_service = MouseService::new();
    app.manage(mouse_service);
    info!("✅ Mouse Service initialized and managed");

    // Hide the main window initially (it will be shown via system tray)
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.hide()?;
        main_window.set_title("Mouseless - 设置")?;
        info!("🪟 Main window initialized (hidden)");
    } else {
        warn!("⚠️ Main window not found");
    }

    // Create system tray
    info!("🔧 Setting up system tray...");
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示设置", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏设置", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show, &hide, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("Mouseless - 键盘鼠标控制")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            handle_tray_event(tray.app_handle(), event);
        })
        .build(app)?;

    info!("✅ System tray created");

    // Check accessibility permissions on startup
    check_macos_permissions(&app_handle);

    info!("🎉 Tauri application setup complete");
    Ok(())
}

fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: tauri::tray::MouseButton::Left,
            button_state: tauri::tray::MouseButtonState::Up,
            ..
        } => {
            // Left click to show/hide main window
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        TrayIconEvent::Enter { .. } => {
            // Optional: Handle mouse enter event
        }
        TrayIconEvent::Leave { .. } => {
            // Optional: Handle mouse leave event
        }
        _ => {}
    }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "quit" => {
            info!("Quitting application via system tray");
            app.exit(0);
        }
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        _ => {}
    }
}

fn check_macos_permissions(_app_handle: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        //TODO: Implement more robust accessibility permission checking
        //TODO: Use proper macOS APIs instead of osascript
        //TODO: Provide detailed permission status information
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
                //TODO: Show user-friendly permission request dialog
                //TODO: Provide step-by-step permission setup guide
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
