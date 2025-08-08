use mouseless_core::{GridConfig, PredictionTarget, AnimationType};
use tauri::{AppHandle, State};
use tracing::{debug, error, info};
use std::sync::{Arc, Mutex};
use crate::ui_manager::UIManager;

type UIManagerState = Arc<Mutex<Option<UIManager>>>;

/// Show grid overlay with specified configuration
#[tauri::command]
pub async fn show_grid_overlay(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
    rows: u32,
    columns: u32,
    show_labels: bool,
    cell_padding: Option<u32>,
    border_width: Option<u32>,
    opacity: Option<f32>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: show_grid_overlay");
    
    let grid_config = GridConfig {
        rows,
        columns,
        show_labels,
        animation_style: AnimationType::Smooth,
        cell_padding: cell_padding.unwrap_or(2),
        border_width: border_width.unwrap_or(1),
        opacity: opacity.unwrap_or(0.8),
    };
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.show_grid_overlay(grid_config).await
            .map_err(|e| format!("Failed to show grid overlay: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Show area overlay with 9-region division
#[tauri::command]
pub async fn show_area_overlay(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: show_area_overlay");
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.show_area_overlay().await
            .map_err(|e| format!("Failed to show area overlay: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Show prediction targets with confidence indicators
#[tauri::command]
pub async fn show_prediction_targets(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
    targets: Vec<PredictionTarget>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: show_prediction_targets with {} targets", targets.len());
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.show_prediction_targets(targets).await
            .map_err(|e| format!("Failed to show prediction targets: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Hide all overlay windows
#[tauri::command]
pub async fn hide_all_overlays(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: hide_all_overlays");
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.hide_all_overlays().await
            .map_err(|e| format!("Failed to hide overlays: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Check if accessibility permissions are granted (macOS specific)
#[tauri::command]
pub async fn check_accessibility_permissions() -> std::result::Result<bool, String> {
    debug!("Tauri command: check_accessibility_permissions");
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Try to access System Events to check accessibility permissions
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of every process")
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    info!("Accessibility permissions are granted");
                    Ok(true)
                } else {
                    info!("Accessibility permissions are not granted");
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Failed to check accessibility permissions: {}", e);
                Ok(false)
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, assume permissions are available
        Ok(true)
    }
}

/// Show activation indicator
#[tauri::command]
pub async fn show_activation_indicator(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: show_activation_indicator");
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.show_activation_indicator().await
            .map_err(|e| format!("Failed to show activation indicator: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Hide activation indicator
#[tauri::command]
pub async fn hide_activation_indicator(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
) -> std::result::Result<(), String> {
    debug!("Tauri command: hide_activation_indicator");
    
    // Extract the UI manager from the state without holding the lock across await
    let ui_manager = {
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        
        ui_manager_guard.take()
    };
    
    if let Some(mut ui_manager) = ui_manager {
        let result = ui_manager.hide_activation_indicator().await
            .map_err(|e| format!("Failed to hide activation indicator: {}", e));
        
        // Put the UI manager back
        let mut ui_manager_guard = ui_manager_state.lock()
            .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
        *ui_manager_guard = Some(ui_manager);
        
        result
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Get grid cell position by key combination
#[tauri::command]
pub async fn get_grid_cell_position(
    _app_handle: AppHandle,
    ui_manager_state: State<'_, UIManagerState>,
    key_combination: String,
) -> std::result::Result<Option<(i32, i32)>, String> {
    debug!("Tauri command: get_grid_cell_position for keys: {}", key_combination);
    
    let ui_manager_guard = ui_manager_state.lock()
        .map_err(|e| format!("Failed to lock UI manager: {}", e))?;
    
    if let Some(ui_manager) = ui_manager_guard.as_ref() {
        if let Some(position) = ui_manager.get_grid_cell_position(&key_combination) {
            Ok(Some((position.x, position.y)))
        } else {
            Ok(None)
        }
    } else {
        Err("UI manager not initialized".to_string())
    }
}

/// Request accessibility permissions (macOS specific)
#[tauri::command]
pub async fn request_accessibility_permissions() -> std::result::Result<(), String> {
    debug!("Tauri command: request_accessibility_permissions");
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Open System Preferences to the Security & Privacy pane
        let result = Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .output();
            
        match result {
            Ok(_) => {
                info!("Opened System Preferences for accessibility permissions");
                Ok(())
            }
            Err(e) => {
                error!("Failed to open System Preferences: {}", e);
                Err(format!("Failed to open System Preferences: {}", e))
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, no action needed
        Ok(())
    }
}