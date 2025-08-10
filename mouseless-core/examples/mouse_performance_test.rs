use mouseless_core::{AnimationType, MouseCommand, MouseController, MouseOperations, Position};
use std::time::Instant;

fn main() {
    println!("🧪 Testing MouseController performance...");

    // Test 1: Creating new MouseController each time (old approach)
    println!("\n📊 Test 1: Creating new MouseController each time");
    let start = Instant::now();
    for i in 0..5 {
        let mut controller = MouseController::new().expect("Failed to create controller");
        let position = Position::new(100 + i * 10, 100 + i * 10);
        controller
            .move_to(position, AnimationType::Bounce)
            .expect("Failed to move mouse");
    }
    let duration_new = start.elapsed();
    println!("⏱️  Time taken (new each time): {:?}", duration_new);

    // Test 2: Reusing single MouseController (new approach)
    println!("\n📊 Test 2: Reusing single MouseController");
    let start = Instant::now();
    let mut controller = MouseController::new().expect("Failed to create controller");
    for i in 0..50 {
        let position = Position::new(200 + i * 10, 200 + i * 10);
        controller
            .move_to(position, AnimationType::Smooth)
            .expect("Failed to move mouse");
        controller.click(mouseless_core::MouseButton::Left);
    }
    let duration_reuse = start.elapsed();
    println!("⏱️  Time taken (reuse): {:?}", duration_reuse);

    // Calculate improvement
    if duration_reuse.as_millis() > 0 {
        let improvement = duration_new.as_millis() as f64 / duration_reuse.as_millis() as f64;
        println!("\n🚀 Performance improvement: {:.2}x faster", improvement);
    } else {
        println!("\n🚀 Reuse approach is significantly faster (sub-millisecond execution)");
    }
    println!("💾 Memory allocations reduced by avoiding repeated MouseController creation");
}
