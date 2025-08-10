use mouseless_core::MouseService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖱️ MouseService Demo - Testing optimized mouse control");

    // Create a single MouseService instance
    let mouse_service = MouseService::new();
    
    println!("📍 Moving mouse in a square pattern...");
    
    // Move mouse in a square pattern to demonstrate reuse
    let positions = [
        (100, 100),
        (300, 100),
        (300, 300),
        (100, 300),
        (100, 100),
    ];
    
    for (i, (x, y)) in positions.iter().enumerate() {
        println!("🎯 Step {}: Moving to ({}, {})", i + 1, x, y);
        
        match mouse_service.move_to_position(*x, *y).await {
            Ok(_) => println!("✅ Successfully moved to ({}, {})", x, y),
            Err(e) => println!("❌ Failed to move mouse: {}", e),
        }
        
        // Small delay between movements
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("🎉 Demo completed! MouseController was reused for all operations.");
    println!("💡 This demonstrates the performance optimization - no repeated initialization overhead.");
    
    Ok(())
}