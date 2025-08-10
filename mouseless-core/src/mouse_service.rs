use crate::{AnimationType, MouseController, MouseOperations, Position};
use std::sync::mpsc;
use std::thread;
use tracing::{error, info};

/// Commands that can be sent to the mouse service
#[derive(Debug)]
pub enum MouseCommand {
    MoveTo {
        x: i32,
        y: i32,
        response_tx: mpsc::Sender<Result<(), String>>,
    },
    RefreshScreens {
        response_tx: mpsc::Sender<Result<(), String>>,
    },
}

/// Thread-safe mouse service that runs MouseController in a dedicated thread
pub struct MouseService {
    command_tx: mpsc::Sender<MouseCommand>,
}

impl MouseService {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<MouseCommand>();

        // Spawn a dedicated thread for mouse operations
        thread::spawn(move || {
            let mut controller: Option<MouseController> = None;

            for command in command_rx {
                match command {
                    MouseCommand::MoveTo { x, y, response_tx } => {
                        info!("🖱️ Moving mouse to position ({}, {})", x, y);

                        // Get or create controller
                        if controller.is_none() {
                            info!("🖱️ Creating new MouseController instance");
                            match MouseController::new() {
                                Ok(new_controller) => {
                                    controller = Some(new_controller);
                                }
                                Err(e) => {
                                    let error_msg = format!("Failed to create mouse controller: {}", e);
                                    error!("❌ {}", error_msg);
                                    let _ = response_tx.send(Err(error_msg));
                                    continue;
                                }
                            }
                        }

                        // Use the controller
                        let result = if let Some(ref mut ctrl) = controller {
                            let position = Position::new(x, y);
                            ctrl.move_to(position, AnimationType::Smooth)
                                .map_err(|e| format!("Failed to move mouse: {}", e))
                        } else {
                            Err("Mouse controller not available".to_string())
                        };

                        match result {
                            Ok(_) => {
                                info!("✅ Mouse moved successfully to ({}, {})", x, y);
                                let _ = response_tx.send(Ok(()));
                            }
                            Err(e) => {
                                error!("❌ Failed to move mouse: {}", e);
                                let _ = response_tx.send(Err(e));
                            }
                        }
                    }
                    MouseCommand::RefreshScreens { response_tx } => {
                        let result = if let Some(ref mut ctrl) = controller {
                            ctrl.refresh_screens()
                                .map_err(|e| format!("Failed to refresh mouse controller screens: {}", e))
                        } else {
                            Err("Mouse controller not available".to_string())
                        };

                        match result {
                            Ok(_) => {
                                info!("🖱️ Mouse controller screen information refreshed");
                                let _ = response_tx.send(Ok(()));
                            }
                            Err(e) => {
                                error!("❌ Failed to refresh screens: {}", e);
                                let _ = response_tx.send(Err(e));
                            }
                        }
                    }
                }
            }
        });

        Self { command_tx }
    }

    /// Move mouse to specific position
    pub async fn move_to_position(&self, x: i32, y: i32) -> Result<(), String> {
        let (response_tx, response_rx) = mpsc::channel();
        
        self.command_tx
            .send(MouseCommand::MoveTo { x, y, response_tx })
            .map_err(|e| format!("Failed to send mouse command: {}", e))?;

        response_rx
            .recv()
            .map_err(|e| format!("Failed to receive mouse response: {}", e))?
    }

    /// Refresh screen information for the mouse controller
    pub async fn refresh_screens(&self) -> Result<(), String> {
        let (response_tx, response_rx) = mpsc::channel();
        
        self.command_tx
            .send(MouseCommand::RefreshScreens { response_tx })
            .map_err(|e| format!("Failed to send refresh command: {}", e))?;

        response_rx
            .recv()
            .map_err(|e| format!("Failed to receive refresh response: {}", e))?
    }
}

impl Clone for MouseService {
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
        }
    }
}

impl Default for MouseService {
    fn default() -> Self {
        Self::new()
    }
}