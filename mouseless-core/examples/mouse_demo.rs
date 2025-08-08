use mouseless_core::{
    mouse::MouseController,
    models::{Position, AnimationType, MovementSpeed, MouseButton},
    traits::MouseOperations,
    init,
};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the core library
    init()?;

    println!("🖱️  Mouseless Enhanced Animation Demo");
    println!("This demo showcases the new smooth cursor movement animations.");
    println!("Press Ctrl+C to stop at any time.\n");

    // Create mouse controller
    let mut controller = MouseController::new()?;
    
    // Get current position
    let start_pos = controller.get_current_position()?;
    println!("Starting position: ({}, {})", start_pos.x, start_pos.y);

    // Demo 1: Enhanced movement speeds with performance tracking
    println!("\n📍 Demo 1: Enhanced movement speeds");
    
    // Slow movement with smooth easing
    controller.set_movement_speed(MovementSpeed::Slow);
    println!("Moving slowly with smooth easing to (200, 200)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(200, 200), AnimationType::Smooth)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Normal movement with bounce
    controller.set_movement_speed(MovementSpeed::Normal);
    println!("Moving normally with bounce to (400, 200)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(400, 200), AnimationType::Bounce)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Fast movement with linear
    controller.set_movement_speed(MovementSpeed::Fast);
    println!("Moving fast with linear to (600, 200)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(600, 200), AnimationType::Linear)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Demo 2: Enhanced animation types with easing functions
    println!("\n🎬 Demo 2: Enhanced animation types");
    
    // Linear animation (no easing)
    println!("Linear animation (no easing) to (200, 400)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(200, 400), AnimationType::Linear)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Smooth animation with ease-out cubic
    println!("Smooth animation (ease-out cubic) to (400, 400)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(400, 400), AnimationType::Smooth)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Bounce animation with enhanced bounce easing
    println!("Bounce animation (enhanced bounce) to (600, 400)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(600, 400), AnimationType::Bounce)?;
    println!("  Completed in: {}ms", start_time.elapsed().as_millis());
    thread::sleep(Duration::from_millis(500));

    // Instant movement (performance test)
    println!("Instant movement (performance test) to (400, 300)...");
    let start_time = std::time::Instant::now();
    controller.move_to(Position::new(400, 300), AnimationType::Instant)?;
    let instant_duration = start_time.elapsed();
    println!("  Completed in: {}μs (should be sub-millisecond)", instant_duration.as_micros());
    thread::sleep(Duration::from_millis(500));

    // Demo 3: Performance demonstration - rapid movements
    println!("\n⚡ Demo 3: Performance demonstration");
    controller.set_movement_speed(MovementSpeed::Fast);
    
    let positions = vec![
        Position::new(300, 300),
        Position::new(500, 300),
        Position::new(500, 500),
        Position::new(300, 500),
        Position::new(400, 400),
    ];

    println!("Performing rapid smooth movements (should maintain sub-10ms per step):");
    for (i, pos) in positions.iter().enumerate() {
        let start_time = std::time::Instant::now();
        controller.move_to(*pos, AnimationType::Smooth)?;
        let duration = start_time.elapsed();
        println!("  Move {}: {}ms", i + 1, duration.as_millis());
        thread::sleep(Duration::from_millis(200));
    }

    // Demo 4: Relative movement with different speeds
    println!("\n↗️  Demo 4: Enhanced relative movement");
    
    // Test relative movement with different speeds
    let speeds = [MovementSpeed::Slow, MovementSpeed::Normal, MovementSpeed::Fast];
    for (i, speed) in speeds.iter().enumerate() {
        controller.set_movement_speed(*speed);
        println!("Relative move with {:?} speed (right 60, down 30)", speed);
        let start_time = std::time::Instant::now();
        controller.move_relative(60, 30)?;
        println!("  Completed in: {}ms", start_time.elapsed().as_millis());
        thread::sleep(Duration::from_millis(300));
    }

    // Demo 5: Animation interpolation showcase
    println!("\n🎯 Demo 5: Animation interpolation showcase");
    
    // Create a complex path using different animation types
    let path = vec![
        (Position::new(200, 200), AnimationType::Linear, "Linear start"),
        (Position::new(600, 200), AnimationType::Smooth, "Smooth horizontal"),
        (Position::new(600, 500), AnimationType::Bounce, "Bounce vertical"),
        (Position::new(200, 500), AnimationType::Smooth, "Smooth return"),
        (Position::new(400, 350), AnimationType::Linear, "Linear to center"),
    ];

    controller.set_movement_speed(MovementSpeed::Normal);
    for (pos, anim_type, description) in path {
        println!("{}: {:?} to ({}, {})", description, anim_type, pos.x, pos.y);
        let start_time = std::time::Instant::now();
        controller.move_to(pos, anim_type)?;
        println!("  Completed in: {}ms", start_time.elapsed().as_millis());
        thread::sleep(Duration::from_millis(400));
    }

    // Demo 6: Click operations with positioning
    println!("\n🖱️  Demo 6: Precise click operations");
    
    // Move to specific position and click
    let click_pos = Position::new(400, 300);
    println!("Moving to click position ({}, {}) with smooth animation...", click_pos.x, click_pos.y);
    controller.move_to(click_pos, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(200));
    
    println!("Performing left click");
    controller.click(MouseButton::Left)?;
    thread::sleep(Duration::from_millis(300));

    println!("Performing right click");
    controller.click(MouseButton::Right)?;
    thread::sleep(Duration::from_millis(300));

    // Demo 7: Screen information and multi-monitor support
    println!("\n🖥️  Demo 7: Screen information");
    let screens = controller.get_screen_bounds()?;
    println!("Detected {} screen(s):", screens.len());
    for screen in &screens {
        println!("  Screen {}: {}x{} at ({}, {}) - Primary: {}", 
                 screen.id, screen.width, screen.height, 
                 screen.x, screen.y, screen.is_primary);
    }

    // Move to center of each screen with smooth animation
    for screen in &screens {
        let center = screen.center();
        println!("Moving to center of screen {} with smooth animation: ({}, {})", 
                 screen.id, center.x, center.y);
        let start_time = std::time::Instant::now();
        controller.move_to(center, AnimationType::Smooth)?;
        println!("  Completed in: {}ms", start_time.elapsed().as_millis());
        thread::sleep(Duration::from_millis(800));
    }

    // Return to starting position with final smooth animation
    println!("\n🏠 Returning to starting position with smooth animation...");
    let start_time = std::time::Instant::now();
    controller.move_to(start_pos, AnimationType::Smooth)?;
    println!("  Final movement completed in: {}ms", start_time.elapsed().as_millis());

    println!("\n✅ Enhanced animation demo completed successfully!");
    println!("🎯 All movements should have maintained sub-10ms step performance.");
    println!("📊 Check the logs for detailed performance metrics.");
    
    Ok(())
}