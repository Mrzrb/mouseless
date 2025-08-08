use mouseless_core::{init, AppInfo, Result};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the core library (logging, etc.)
    init()?;

    let app_info = AppInfo::default();
    info!(
        "Starting {} v{}",
        app_info.name,
        app_info.version
    );

    // TODO: Initialize Tauri application in future tasks
    // TODO: Set up input handling in future tasks
    // TODO: Set up mouse control in future tasks
    // TODO: Set up UI rendering in future tasks

    info!("Application setup complete - ready for implementation");
    
    // For now, just keep the application running
    tokio::signal::ctrl_c().await.map_err(|e| {
        error!("Failed to listen for ctrl+c signal: {}", e);
        mouseless_core::MouselessError::SystemError(e)
    })?;

    info!("Shutting down application");
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