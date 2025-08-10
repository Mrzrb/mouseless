use mouseless_core::{
    traits::ModeController, Action, InteractionMode, KeyBindings, KeyInput, ModeManager,
};
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    mouseless_core::init()?;

    //TODO: Add demo for configuration file loading
    //TODO: Add demo for multi-screen grid functionality
    //TODO: Add demo for prediction mode when implemented
    //TODO: Add interactive demo with real keyboard input

    println!("🎯 Mouseless Mode Management System Demo");
    println!("========================================");

    // Create a mode manager with default key bindings
    let mut manager = ModeManager::new(KeyBindings::default());

    // Subscribe to mode events
    let mut event_receiver = manager.subscribe_to_events();

    println!("\n📋 Initial State:");
    println!("  Active: {}", manager.is_active());
    println!("  Current Mode: {:?}", manager.get_current_mode());
    println!("  History: {:?}", manager.get_mode_history());

    // Activate basic mode
    println!("\n🔄 Activating Basic Mode...");
    manager.activate_mode(InteractionMode::Basic).await?;

    // Check for event
    if let Ok(event) = event_receiver.try_recv() {
        println!("  📢 Event: {:?}", event);
    }

    println!("  Active: {}", manager.is_active());
    println!("  Current Mode: {:?}", manager.get_current_mode());

    // Test some basic input processing
    println!("\n⌨️  Testing Basic Input Processing:");

    let test_inputs = vec![
        ('i', "Move Up"),
        ('k', "Move Down"),
        ('j', "Move Left"),
        ('l', "Move Right"),
        ('n', "Left Click"),
        ('m', "Right Click"),
        ('u', "Scroll Up"),
        ('o', "Scroll Down"),
        ('f', "Toggle Speed"),
        ('b', "Toggle Hold"),
        ('g', "Switch to Grid Mode"),
    ];

    for (key, description) in test_inputs {
        let input = KeyInput {
            key,
            modifiers: vec![],
            timestamp: SystemTime::now(),
        };

        let action = manager.handle_input(input).await?;
        println!("  {} ('{}'): {:?}", description, key, action);

        // Check for events after mode switching
        if matches!(action, Action::ActivateMode(_)) {
            if let Ok(event) = event_receiver.try_recv() {
                println!("    📢 Event: {:?}", event);
            }
        }
    }

    println!("\n📊 Final State:");
    println!("  Active: {}", manager.is_active());
    println!("  Current Mode: {:?}", manager.get_current_mode());
    println!("  History: {:?}", manager.get_mode_history());
    println!("  Movement Speed: {:.1}", manager.get_movement_speed());
    println!("  Holding: {}", manager.is_holding());

    // Test mode switching
    println!("\n🔄 Testing Mode Switching:");
    manager.activate_mode(InteractionMode::Area).await?;
    println!("  Switched to Area Mode");

    manager.activate_mode(InteractionMode::Prediction).await?;
    println!("  Switched to Prediction Mode");

    println!("  Final History: {:?}", manager.get_mode_history());

    // Deactivate
    println!("\n🔄 Deactivating Current Mode...");
    manager.deactivate_current_mode().await?;

    if let Ok(event) = event_receiver.try_recv() {
        println!("  📢 Event: {:?}", event);
    }

    println!("  Active: {}", manager.is_active());
    println!("  Current Mode: {:?}", manager.get_current_mode());

    println!("\n✅ Mode Management System Demo Complete!");

    Ok(())
}
