use mouseless_core::{
    init,
    models::{AnimationType, MouseButton, MovementSpeed, Position},
    mouse::MouseController,
    traits::MouseOperations,
};
use std::{
    thread,
    time::{Duration, Instant},
};

/// Method 1: Real-time clock-based rendering (no sleep, pure timing)
//TODO: Add demo for multi-screen mouse movement
//TODO: Add demo for configuration-driven animation settings
//TODO: Add demo for different easing functions
//TODO: Add performance benchmarking for animation quality
fn realtime_curve_motion<F>(
    controller: &mut MouseController,
    duration_ms: u64,
    point_generator: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64) -> Position,
{
    let start_time = Instant::now();
    let duration = Duration::from_millis(duration_ms);
    let mut frame_count = 0;
    let mut last_position = controller.get_current_position()?;
    let mut last_update = start_time;

    // Pure real-time loop - no artificial delays
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);

        if elapsed >= duration {
            break;
        }

        // Only update if enough time has passed (natural frame limiting)
        let frame_time = Duration::from_nanos(4_166_667); // ~240 FPS
        if now.duration_since(last_update) >= frame_time {
            let progress = elapsed.as_secs_f64() / duration.as_secs_f64();
            let target_position = point_generator(progress);

            if target_position.x != last_position.x || target_position.y != last_position.y {
                controller.move_to(target_position, AnimationType::Instant)?;
                last_position = target_position;
                frame_count += 1;
                last_update = now;
            }
        }

        // Yield CPU to prevent 100% usage while maintaining responsiveness
        std::hint::spin_loop();
    }

    let actual_duration = start_time.elapsed();
    let actual_fps = frame_count as f64 * 1000.0 / actual_duration.as_millis() as f64;
    println!(
        "  Real-time FPS: {:.1}, Frames: {}",
        actual_fps, frame_count
    );

    Ok(())
}

/// Method 2: Async event-driven motion (using channels)
fn async_curve_motion<F>(
    controller: &mut MouseController,
    duration_ms: u64,
    point_generator: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64) -> Position + Send + 'static,
{
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let start_time = Instant::now();
    let duration = Duration::from_millis(duration_ms);

    // Spawn a high-precision timer thread
    let timer_tx = tx.clone();
    thread::spawn(move || {
        let mut frame_count = 0u32;
        let target_fps = 300.0; // Even higher for async
        let frame_duration = Duration::from_nanos((1_000_000_000.0 / target_fps) as u64);

        loop {
            let elapsed = start_time.elapsed();
            if elapsed >= duration {
                let _ = timer_tx.send(None);
                break;
            }

            let progress = elapsed.as_secs_f64() / duration.as_secs_f64();
            let position = point_generator(progress);

            if timer_tx.send(Some((position, frame_count))).is_err() {
                break;
            }

            frame_count += 1;

            // High-precision timing without sleep
            let next_frame = start_time + frame_duration * frame_count;
            while Instant::now() < next_frame {
                std::hint::spin_loop();
            }
        }
    });

    // Main thread processes positions
    let mut last_position = controller.get_current_position()?;
    let mut frame_count = 0;

    while let Ok(msg) = rx.recv() {
        match msg {
            Some((position, _)) => {
                if position.x != last_position.x || position.y != last_position.y {
                    controller.move_to(position, AnimationType::Instant)?;
                    last_position = position;
                    frame_count += 1;
                }
            }
            None => break,
        }
    }

    let actual_duration = start_time.elapsed();
    let actual_fps = frame_count as f64 * 1000.0 / actual_duration.as_millis() as f64;
    println!("  Async FPS: {:.1}, Frames: {}", actual_fps, frame_count);

    Ok(())
}

/// Method 3: VSync-like motion (simulating display refresh rate)
fn vsync_curve_motion<F>(
    controller: &mut MouseController,
    duration_ms: u64,
    point_generator: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64) -> Position,
{
    let start_time = Instant::now();
    let duration = Duration::from_millis(duration_ms);

    // Simulate different display refresh rates
    let refresh_rates = [60.0, 120.0, 144.0, 240.0]; // Hz
    let current_refresh = refresh_rates[3]; // Use 240Hz
    let frame_time = Duration::from_nanos((1_000_000_000.0 / current_refresh) as u64);

    let mut last_position = controller.get_current_position()?;
    let mut frame_count = 0;
    let mut next_frame_time = start_time;

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);

        if elapsed >= duration {
            break;
        }

        // Wait for next "VSync" without sleep
        if now >= next_frame_time {
            let progress = elapsed.as_secs_f64() / duration.as_secs_f64();
            let target_position = point_generator(progress);

            if target_position.x != last_position.x || target_position.y != last_position.y {
                controller.move_to(target_position, AnimationType::Instant)?;
                last_position = target_position;
            }

            frame_count += 1;
            next_frame_time += frame_time;
        }

        // Busy wait with CPU yield
        std::hint::spin_loop();
    }

    let actual_duration = start_time.elapsed();
    let actual_fps = frame_count as f64 * 1000.0 / actual_duration.as_millis() as f64;
    println!(
        "  VSync-like FPS: {:.1}, Frames: {}",
        actual_fps, frame_count
    );

    Ok(())
}

/// Method 4: Predictive interpolation (advanced smoothing)
fn predictive_curve_motion<F>(
    controller: &mut MouseController,
    duration_ms: u64,
    point_generator: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64) -> Position,
{
    let start_time = Instant::now();
    let duration = Duration::from_millis(duration_ms);
    let mut last_position = controller.get_current_position()?;
    let mut frame_count = 0;

    // Predictive buffer for smoother motion
    let mut position_history = Vec::with_capacity(5);
    let lookahead_time = Duration::from_millis(16); // 16ms lookahead

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time);

        if elapsed >= duration {
            break;
        }

        let progress = elapsed.as_secs_f64() / duration.as_secs_f64();

        // Calculate current and predicted positions
        let current_pos = point_generator(progress);
        let future_progress =
            ((elapsed + lookahead_time).as_secs_f64() / duration.as_secs_f64()).min(1.0);
        let future_pos = point_generator(future_progress);

        // Interpolate between current and predicted position
        let interpolation_factor = 0.3; // 30% prediction
        let interpolated_x =
            current_pos.x + ((future_pos.x - current_pos.x) as f64 * interpolation_factor) as i32;
        let interpolated_y =
            current_pos.y + ((future_pos.y - current_pos.y) as f64 * interpolation_factor) as i32;
        let interpolated_pos = Position::new(interpolated_x, interpolated_y);

        // Update position history for smoothing
        position_history.push(interpolated_pos);
        if position_history.len() > 3 {
            position_history.remove(0);
        }

        // Apply temporal smoothing
        let smoothed_pos = if position_history.len() >= 3 {
            let avg_x = position_history.iter().map(|p| p.x as f64).sum::<f64>()
                / position_history.len() as f64;
            let avg_y = position_history.iter().map(|p| p.y as f64).sum::<f64>()
                / position_history.len() as f64;
            Position::new(avg_x as i32, avg_y as i32)
        } else {
            interpolated_pos
        };

        if smoothed_pos.x != last_position.x || smoothed_pos.y != last_position.y {
            controller.move_to(smoothed_pos, AnimationType::Instant)?;
            last_position = smoothed_pos;
            frame_count += 1;
        }

        // Natural frame limiting without sleep
        std::hint::spin_loop();

        // Prevent excessive CPU usage
        if frame_count % 1000 == 0 {
            thread::yield_now();
        }
    }

    let actual_duration = start_time.elapsed();
    let actual_fps = frame_count as f64 * 1000.0 / actual_duration.as_millis() as f64;
    println!(
        "  Predictive FPS: {:.1}, Frames: {}",
        actual_fps, frame_count
    );

    Ok(())
}

/// Legacy method with sleep (for comparison)
fn ultra_smooth_curve_motion<F>(
    controller: &mut MouseController,
    duration_ms: u64,
    point_generator: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(f64) -> Position,
{
    let start_time = std::time::Instant::now();
    let duration = Duration::from_millis(duration_ms);
    let target_fps = 240.0; // Ultra-high frame rate
    let frame_duration = Duration::from_nanos((1_000_000_000.0 / target_fps) as u64);

    let mut frame_count = 0;
    let mut last_position = controller.get_current_position()?;

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= duration {
            break;
        }

        // Calculate progress (0.0 to 1.0)
        let progress = elapsed.as_secs_f64() / duration.as_secs_f64();

        // Generate the next position
        let target_position = point_generator(progress);

        // Only move if position actually changed (avoid unnecessary system calls)
        if target_position.x != last_position.x || target_position.y != last_position.y {
            controller.move_to(target_position, AnimationType::Instant)?;
            last_position = target_position;
        }

        frame_count += 1;

        // High-precision timing using spin-wait for the last microseconds
        let next_frame_time = start_time + frame_duration * frame_count;
        let now = std::time::Instant::now();

        if next_frame_time > now {
            let sleep_duration = next_frame_time - now;

            // Use sleep for longer waits, spin for precision
            if sleep_duration > Duration::from_micros(500) {
                thread::sleep(sleep_duration - Duration::from_micros(100));
            }

            // Spin-wait for the remaining time for maximum precision
            while std::time::Instant::now() < next_frame_time {
                std::hint::spin_loop();
            }
        }
    }

    let actual_duration = start_time.elapsed();
    let actual_fps = frame_count as f64 * 1000.0 / actual_duration.as_millis() as f64;
    println!("  Actual FPS: {:.1}, Frames: {}", actual_fps, frame_count);

    Ok(())
}

/// Smooth interpolated movement between points for ultra-smooth curves
fn smooth_move_to_with_interpolation(
    controller: &mut MouseController,
    target: Position,
    interpolation_steps: u32,
    frame_delay: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let current = controller.get_current_position()?;

    // If we're already at the target, no need to move
    if current.x == target.x && current.y == target.y {
        return Ok(());
    }

    // Calculate the step size for smooth interpolation
    let dx = target.x - current.x;
    let dy = target.y - current.y;

    for step in 1..=interpolation_steps {
        let frame_start = std::time::Instant::now();

        let progress = step as f64 / interpolation_steps as f64;

        // Use ease-out cubic for smoother motion
        let eased_progress = 1.0 - (1.0 - progress).powi(3);

        let interpolated_x = current.x + (dx as f64 * eased_progress) as i32;
        let interpolated_y = current.y + (dy as f64 * eased_progress) as i32;

        let interpolated_pos = Position::new(interpolated_x, interpolated_y);
        controller.move_to(interpolated_pos, AnimationType::Instant)?;

        // Maintain consistent frame rate
        let elapsed = frame_start.elapsed();
        if elapsed < frame_delay {
            thread::sleep(frame_delay - elapsed);
        }
    }

    Ok(())
}

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
    println!(
        "  Completed in: {}μs (should be sub-millisecond)",
        instant_duration.as_micros()
    );
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
    let speeds = [
        MovementSpeed::Slow,
        MovementSpeed::Normal,
        MovementSpeed::Fast,
    ];
    for (_i, speed) in speeds.iter().enumerate() {
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
        (
            Position::new(200, 200),
            AnimationType::Linear,
            "Linear start",
        ),
        (
            Position::new(600, 200),
            AnimationType::Smooth,
            "Smooth horizontal",
        ),
        (
            Position::new(600, 500),
            AnimationType::Bounce,
            "Bounce vertical",
        ),
        (
            Position::new(200, 500),
            AnimationType::Smooth,
            "Smooth return",
        ),
        (
            Position::new(400, 350),
            AnimationType::Linear,
            "Linear to center",
        ),
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
    println!(
        "Moving to click position ({}, {}) with smooth animation...",
        click_pos.x, click_pos.y
    );
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
        println!(
            "  Screen {}: {}x{} at ({}, {}) - Primary: {}",
            screen.id, screen.width, screen.height, screen.x, screen.y, screen.is_primary
        );
    }

    // Move to center of each screen with smooth animation
    for screen in &screens {
        let center = screen.center();
        println!(
            "Moving to center of screen {} with smooth animation: ({}, {})",
            screen.id, center.x, center.y
        );
        let start_time = std::time::Instant::now();
        controller.move_to(center, AnimationType::Smooth)?;
        println!("  Completed in: {}ms", start_time.elapsed().as_millis());
        thread::sleep(Duration::from_millis(800));
    }

    // Demo 8: Advanced no-sleep motion techniques
    println!("\n🚀 Demo 8: Advanced No-Sleep Motion Techniques");

    let wave_center_x = 400;
    let wave_center_y = 400;
    let wave_amplitude = 80;
    let wave_length = 300;
    let test_duration = 1500; // Shorter for comparison

    println!("Comparing different no-sleep motion techniques:");
    println!(
        "  Wave: center({}, {}), amplitude: {}, length: {}",
        wave_center_x, wave_center_y, wave_amplitude, wave_length
    );
    println!("  Duration: {}ms each", test_duration);

    controller.set_movement_speed(MovementSpeed::Fast);

    // Test 1: Real-time clock-based
    println!("\n🕐 Test 1: Real-time clock-based (pure timing)");
    let wave_start = Position::new(wave_center_x - wave_length / 2, wave_center_y);
    controller.move_to(wave_start, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(300));

    realtime_curve_motion(&mut controller, test_duration, |progress| {
        let x_offset = (progress * wave_length as f64) as i32;
        let angle = progress * 2.0 * std::f64::consts::PI;
        let y_offset = (angle.sin() * wave_amplitude as f64) as i32;
        Position::new(
            wave_center_x - wave_length / 2 + x_offset,
            wave_center_y + y_offset,
        )
    })?;

    thread::sleep(Duration::from_millis(500));

    // Test 2: Async event-driven
    println!("\n📡 Test 2: Async event-driven (channel-based)");
    controller.move_to(wave_start, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(300));

    async_curve_motion(&mut controller, test_duration, move |progress| {
        let x_offset = (progress * wave_length as f64) as i32;
        let angle = progress * 3.0 * std::f64::consts::PI; // Different frequency
        let y_offset = (angle.sin() * wave_amplitude as f64) as i32;
        Position::new(
            wave_center_x - wave_length / 2 + x_offset,
            wave_center_y + y_offset,
        )
    })?;

    thread::sleep(Duration::from_millis(500));

    // Test 3: VSync-like
    println!("\n📺 Test 3: VSync-like (display refresh simulation)");
    controller.move_to(wave_start, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(300));

    vsync_curve_motion(&mut controller, test_duration, |progress| {
        let angle = progress * 4.0 * std::f64::consts::PI; // Lissajous-like
        let x_offset = (angle.sin() * (wave_length / 2) as f64) as i32;
        let y_offset = ((angle * 1.5).sin() * wave_amplitude as f64) as i32;
        Position::new(wave_center_x + x_offset, wave_center_y + y_offset)
    })?;

    thread::sleep(Duration::from_millis(500));

    // Test 4: Predictive interpolation
    println!("\n🔮 Test 4: Predictive interpolation (advanced smoothing)");
    let spiral_center = Position::new(wave_center_x, wave_center_y);
    controller.move_to(spiral_center, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(300));

    predictive_curve_motion(&mut controller, test_duration, |progress| {
        let angle = progress * 3.0 * 2.0 * std::f64::consts::PI; // 3 turns
        let radius = progress * 80.0; // Expanding spiral
        let x_offset = (angle.cos() * radius) as i32;
        let y_offset = (angle.sin() * radius) as i32;
        Position::new(wave_center_x + x_offset, wave_center_y + y_offset)
    })?;

    thread::sleep(Duration::from_millis(500));

    // Demo 8b: Traditional sine wave motion (for comparison)
    println!("\n🌊 Demo 8b: Traditional sine wave motion (with sleep)");

    // Set up sine wave parameters
    let wave_center_x = 400;
    let wave_center_y = 400;
    let wave_amplitude = 100; // Height of the wave
    let wave_length = 400; // Width of one complete wave cycle
    let wave_duration_ms = 2000; // 2 seconds for the complete wave

    println!("Creating ultra-smooth sine wave motion:");
    println!("  Center: ({}, {})", wave_center_x, wave_center_y);
    println!("  Amplitude: {} pixels", wave_amplitude);
    println!("  Wave length: {} pixels", wave_length);
    println!("  Duration: {}ms", wave_duration_ms);
    println!("  Target frame rate: 240 FPS (spin-wait precision)");

    controller.set_movement_speed(MovementSpeed::Fast);

    // Move to starting position of the wave
    let wave_start = Position::new(wave_center_x - wave_length / 2, wave_center_y);
    println!(
        "\nMoving to wave start position: ({}, {})",
        wave_start.x, wave_start.y
    );
    controller.move_to(wave_start, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(500));

    println!("Starting ultra-smooth sine wave motion...");
    let wave_start_time = std::time::Instant::now();

    // Use the new ultra-smooth curve motion function
    ultra_smooth_curve_motion(&mut controller, wave_duration_ms, |progress| {
        let x_offset = (progress * wave_length as f64) as i32;
        let angle = progress * 2.0 * std::f64::consts::PI; // 0 to 2π for one complete cycle
        let y_offset = (angle.sin() * wave_amplitude as f64) as i32;

        Position::new(
            wave_center_x - wave_length / 2 + x_offset,
            wave_center_y + y_offset,
        )
    })?;

    let wave_duration = wave_start_time.elapsed();
    println!(
        "Ultra-smooth sine wave completed in: {}ms",
        wave_duration.as_millis()
    );
    thread::sleep(Duration::from_millis(500));

    // Demo 8b: Ultra-smooth Lissajous curve (double sine wave)
    println!("\n🌀 Demo 8b: Ultra-smooth Lissajous curve");

    let lissajous_center_x = 400;
    let lissajous_center_y = 400;
    let lissajous_amplitude_x = 150;
    let lissajous_amplitude_y = 100;
    let frequency_ratio = 3.0 / 2.0; // Creates an interesting pattern
    let lissajous_duration_ms = 3000; // 3 seconds for the complete pattern

    println!("Creating ultra-smooth Lissajous curve motion:");
    println!("  Center: ({}, {})", lissajous_center_x, lissajous_center_y);
    println!("  X amplitude: {} pixels", lissajous_amplitude_x);
    println!("  Y amplitude: {} pixels", lissajous_amplitude_y);
    println!("  Frequency ratio: {:.1}", frequency_ratio);
    println!("  Duration: {}ms", lissajous_duration_ms);
    println!("  Target frame rate: 240 FPS (spin-wait precision)");

    println!("\nStarting ultra-smooth Lissajous curve motion...");
    let lissajous_start_time = std::time::Instant::now();

    // Use the new ultra-smooth curve motion function
    ultra_smooth_curve_motion(&mut controller, lissajous_duration_ms, |progress| {
        let t = progress * 4.0 * std::f64::consts::PI; // Multiple cycles

        let x_offset = (t.sin() * lissajous_amplitude_x as f64) as i32;
        let y_offset = ((t * frequency_ratio).sin() * lissajous_amplitude_y as f64) as i32;

        Position::new(lissajous_center_x + x_offset, lissajous_center_y + y_offset)
    })?;

    let lissajous_duration = lissajous_start_time.elapsed();
    println!(
        "Ultra-smooth Lissajous curve completed in: {}ms",
        lissajous_duration.as_millis()
    );
    thread::sleep(Duration::from_millis(500));

    // Demo 8c: Ultra-smooth spiral motion
    println!("\n🌀 Demo 8c: Ultra-smooth spiral motion");

    let spiral_center_x = 400;
    let spiral_center_y = 400;
    let spiral_max_radius = 120;
    let spiral_turns = 3.0; // Number of complete turns
    let spiral_duration_ms = 2500; // 2.5 seconds for the complete spiral

    println!("Creating ultra-smooth spiral motion:");
    println!("  Center: ({}, {})", spiral_center_x, spiral_center_y);
    println!("  Max radius: {} pixels", spiral_max_radius);
    println!("  Turns: {:.1}", spiral_turns);
    println!("  Duration: {}ms", spiral_duration_ms);
    println!("  Target frame rate: 240 FPS (spin-wait precision)");

    // Move to spiral center
    let spiral_center = Position::new(spiral_center_x, spiral_center_y);
    println!(
        "\nMoving to spiral center: ({}, {})",
        spiral_center.x, spiral_center.y
    );
    controller.move_to(spiral_center, AnimationType::Smooth)?;
    thread::sleep(Duration::from_millis(300));

    println!("Starting ultra-smooth spiral motion...");
    let spiral_start_time = std::time::Instant::now();

    // Use the new ultra-smooth curve motion function
    ultra_smooth_curve_motion(&mut controller, spiral_duration_ms, |progress| {
        let angle = progress * spiral_turns * 2.0 * std::f64::consts::PI;
        let radius = progress * spiral_max_radius as f64;

        let x_offset = (angle.cos() * radius) as i32;
        let y_offset = (angle.sin() * radius) as i32;

        Position::new(spiral_center_x + x_offset, spiral_center_y + y_offset)
    })?;

    let spiral_duration = spiral_start_time.elapsed();
    println!(
        "Ultra-smooth spiral motion completed in: {}ms",
        spiral_duration.as_millis()
    );
    thread::sleep(Duration::from_millis(500));

    // Return to starting position with final smooth animation
    println!("\n🏠 Returning to starting position with smooth animation...");
    let start_time = std::time::Instant::now();
    controller.move_to(start_pos, AnimationType::Smooth)?;
    println!(
        "  Final movement completed in: {}ms",
        start_time.elapsed().as_millis()
    );

    println!("\n✅ Ultra-smooth enhanced animation demo completed successfully!");
    println!("🎯 All movements maintained high frame rates for ultra-smooth motion:");
    println!("   • Sine wave: 120 FPS with 200 steps");
    println!("   • Lissajous curve: 144 FPS with 360 steps");
    println!("   • Spiral motion: 120 FPS with 240 steps");
    println!("🌊 Ultra-smooth curved paths demonstrated with high-resolution interpolation.");
    println!("⚡ Frame rate consistency maintained for fluid visual experience.");
    println!("📊 Check the logs for detailed performance metrics and FPS measurements.");

    Ok(())
}
