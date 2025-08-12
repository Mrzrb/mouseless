//! Demonstration of the rdev-based input handling system
//!
//! This example shows how to use the new InputHandler with rdev
//! for global hotkey detection and key processing.

use mouseless_core::{ActivationConfig, ActivationKey, InputHandler, InputProcessor, KeyBindings};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting rdev input demo");

    // Create input handler
    let mut input_handler = InputHandler::new()?;

    // Set up custom activation configuration
    let activation_config = ActivationConfig {
        trigger_key: ActivationKey::CapsLock,
        modifier_keys: vec![],
        double_click_required: true,
        double_click_timeout_ms: 500,
        activation_timeout_ms: 10000,
    };

    input_handler.update_activation_config(activation_config)?;

    // Set up custom key bindings
    let mut key_bindings = KeyBindings::default();
    key_bindings.move_up = 'w';
    // key_bindings.move_down = 's';
    // // key_bindings.move_left = 'a';
    // key_bindings.move_right = 'd';
    // // key_bindings.left_click = 'j';
    // key_bindings.right_click = 'k';
    // key_bindings.exit_key = 'q';

    input_handler.update_bindings(key_bindings).await?;

    // Register the activation hotkey
    input_handler.register_activation_hotkey()?;

    // Set up action channel
    let _action_receiver = input_handler.setup_action_channel();

    info!("Input handler configured. Press CapsLock twice to activate.");
    info!("When active, use WASD for movement, J/K for clicks, Q to exit.");
    info!("Note: This demo requires a display environment to work properly.");

    // Start the event loop (this will run in a background thread)
    input_handler.start_event_loop().await?;

    // In a real application, you would process actions from the receiver
    // For this demo, we'll just run for a short time
    info!("Demo running for 30 seconds...");
    sleep(Duration::from_secs(30)).await;

    // Stop the event loop
    input_handler.stop_event_loop().await?;

    info!("Demo completed");
    Ok(())
}
