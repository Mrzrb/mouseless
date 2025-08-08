use std::thread;
use std::time::Instant;
use enigo::{Enigo, Mouse, Settings};
use tracing;
use crate::{
    animation::{AnimationInterpolator, AnimationMetrics},
    error::{MouseError, MouseResult},
    models::{Position, ScreenBounds, MouseButton, ScrollDirection, AnimationType, MovementSpeed},
    traits::MouseOperations,
    screen::ScreenManager,
};

/// Mouse controller implementation using enigo
pub struct MouseController {
    enigo: Enigo,
    current_position: Position,
    movement_speed: MovementSpeed,
    screen_info: Vec<ScreenBounds>,
}

impl MouseController {
    /// Create a new mouse controller instance
    pub fn new() -> MouseResult<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| MouseError::ScreenDetectionFailed {
                reason: format!("Failed to initialize enigo: {}", e),
            })?;

        // Initialize screen information
        let screens = ScreenManager::detect_screens()?;

        // Initialize current position
        let current_pos = Self::get_cursor_position_from_enigo(&enigo)?;

        let controller = Self {
            enigo,
            current_position: current_pos,
            movement_speed: MovementSpeed::Normal,
            screen_info: screens,
        };

        Ok(controller)
    }

    /// Refresh screen information
    pub fn refresh_screens(&mut self) -> MouseResult<()> {
        let screens = ScreenManager::detect_screens()?;
        self.screen_info = screens;
        Ok(())
    }

    /// Move cursor to the center of a specific screen
    pub fn move_to_screen(&mut self, screen_number: u8) -> MouseResult<()> {
        let screen_id = ScreenManager::map_screen_number_to_id(screen_number)?;
        let center = ScreenManager::get_screen_center(screen_id)?;
        self.move_to(center, AnimationType::Smooth)
    }

    /// Move cursor by relative offset from current position
    pub fn move_relative(&mut self, dx: i32, dy: i32) -> MouseResult<()> {
        let current = self.get_current_position()?;
        let speed = self.get_movement_speed();
        let pixel_distance = Self::speed_to_pixels(speed);
        
        let new_position = Position::new(
            current.x + (dx * pixel_distance),
            current.y + (dy * pixel_distance),
        );
        
        self.move_to(new_position, AnimationType::Linear)
    }

    /// Get current cursor position from the system
    fn get_cursor_position_from_enigo(enigo: &Enigo) -> MouseResult<Position> {
        let (x, y) = enigo.location()
            .map_err(|e| MouseError::ScreenDetectionFailed {
                reason: format!("Failed to get cursor position: {}", e),
            })?;
        
        Ok(Position::new(x, y))
    }

    /// Update current position from system
    fn update_current_position(&mut self) -> MouseResult<()> {
        let pos = Self::get_cursor_position_from_enigo(&self.enigo)?;
        self.current_position = pos;
        Ok(())
    }

    /// Convert movement speed to pixel distance
    fn speed_to_pixels(speed: MovementSpeed) -> i32 {
        match speed {
            MovementSpeed::Slow => 5,
            MovementSpeed::Normal => 15,
            MovementSpeed::Fast => 30,
        }
    }

    /// Validate position is within screen bounds
    fn validate_position(&self, position: Position) -> MouseResult<Position> {
        let screens = &self.screen_info;
        
        // If no screen_id specified, use the primary screen
        let target_screen = if let Some(screen_id) = position.screen_id {
            screens.iter().find(|s| s.id == screen_id)
        } else {
            screens.iter().find(|s| s.is_primary)
        };

        let screen = target_screen.ok_or_else(|| MouseError::ScreenDetectionFailed {
            reason: format!("Screen not found for position: {:?}", position),
        })?;

        // Clamp position to screen bounds
        let clamped_x = position.x.max(screen.x).min(screen.x + screen.width as i32 - 1);
        let clamped_y = position.y.max(screen.y).min(screen.y + screen.height as i32 - 1);

        Ok(Position {
            x: clamped_x,
            y: clamped_y,
            screen_id: Some(screen.id),
        })
    }

    /// Perform instant movement without animation
    fn move_instant(&mut self, position: Position) -> MouseResult<()> {
        let validated_pos = self.validate_position(position)?;
        
        self.enigo.move_mouse(validated_pos.x, validated_pos.y, enigo::Coordinate::Abs)
            .map_err(|e| MouseError::MovementFailed {
                x: validated_pos.x,
                y: validated_pos.y,
                reason: format!("Enigo movement failed: {}", e),
            })?;

        // Update current position
        self.current_position = validated_pos;
        
        Ok(())
    }

    /// Perform animated movement with enhanced easing and performance tracking
    fn move_animated(&mut self, target: Position, animation_type: AnimationType) -> MouseResult<()> {
        let current = self.current_position;
        let validated_target = self.validate_position(target)?;
        
        // For instant movement, skip animation
        if animation_type == AnimationType::Instant {
            return self.move_instant(validated_target);
        }

        // Create animation interpolator
        let interpolator = AnimationInterpolator::new(
            current,
            validated_target,
            self.movement_speed,
            animation_type,
        );

        let config = interpolator.config();
        let step_delay = std::time::Duration::from_millis(config.step_delay_ms());
        
        // Track performance metrics
        let mut metrics = AnimationMetrics::new();
        
        // Execute animation steps
        for step in 0..config.steps {
            let step_start = Instant::now();
            
            if let Some(position) = interpolator.next_position(step) {
                self.move_instant(position)?;
                
                // Record step timing for performance monitoring
                let step_duration = step_start.elapsed().as_millis() as u64;
                metrics.record_step(step_duration);
                
                // Sleep for remaining time to maintain consistent timing
                if step < config.steps - 1 && step_delay > std::time::Duration::ZERO {
                    let elapsed = step_start.elapsed();
                    if elapsed < step_delay {
                        thread::sleep(step_delay - elapsed);
                    }
                }
            }
        }

        // Log performance metrics if they don't meet requirements
        if !metrics.meets_performance_requirement() {
            tracing::warn!(
                "Animation performance below requirement: avg={:.2}ms, max={}ms",
                metrics.average_step_time_ms,
                metrics.max_step_time_ms
            );
        } else {
            tracing::debug!(
                "Animation completed successfully: avg={:.2}ms, steps={}",
                metrics.average_step_time_ms,
                metrics.steps_completed
            );
        }

        Ok(())
    }
}

impl MouseOperations for MouseController {
    fn move_to(&mut self, position: Position, animation: AnimationType) -> MouseResult<()> {
        self.move_animated(position, animation)
    }

    fn click(&mut self, button: MouseButton) -> MouseResult<()> {
        
        let enigo_button = match button {
            MouseButton::Left => enigo::Button::Left,
            MouseButton::Right => enigo::Button::Right,
            MouseButton::Middle => enigo::Button::Middle,
        };

        self.enigo.button(enigo_button, enigo::Direction::Click)
            .map_err(|e| MouseError::ClickFailed {
                button: format!("{:?}", button),
                reason: format!("Enigo click failed: {}", e),
            })?;

        Ok(())
    }

    fn scroll(&mut self, direction: ScrollDirection, amount: i32) -> MouseResult<()> {
        
        match direction {
            ScrollDirection::Up => {
                self.enigo.scroll(amount, enigo::Axis::Vertical)
                    .map_err(|e| MouseError::ScrollFailed {
                        direction: "up".to_string(),
                        reason: format!("Enigo scroll failed: {}", e),
                    })?;
            }
            ScrollDirection::Down => {
                self.enigo.scroll(-amount, enigo::Axis::Vertical)
                    .map_err(|e| MouseError::ScrollFailed {
                        direction: "down".to_string(),
                        reason: format!("Enigo scroll failed: {}", e),
                    })?;
            }
            ScrollDirection::Left => {
                self.enigo.scroll(-amount, enigo::Axis::Horizontal)
                    .map_err(|e| MouseError::ScrollFailed {
                        direction: "left".to_string(),
                        reason: format!("Enigo scroll failed: {}", e),
                    })?;
            }
            ScrollDirection::Right => {
                self.enigo.scroll(amount, enigo::Axis::Horizontal)
                    .map_err(|e| MouseError::ScrollFailed {
                        direction: "right".to_string(),
                        reason: format!("Enigo scroll failed: {}", e),
                    })?;
            }
        }

        Ok(())
    }

    fn get_current_position(&self) -> MouseResult<Position> {
        // Try to get the real-time position first
        match Self::get_cursor_position_from_enigo(&self.enigo) {
            Ok(pos) => Ok(pos),
            Err(_) => {
                // Fall back to cached position if real-time detection fails
                Ok(self.current_position)
            }
        }
    }

    fn get_screen_bounds(&self) -> MouseResult<Vec<ScreenBounds>> {
        Ok(self.screen_info.clone())
    }

    fn set_movement_speed(&mut self, speed: MovementSpeed) {
        self.movement_speed = speed;
    }

    fn get_movement_speed(&self) -> MovementSpeed {
        self.movement_speed
    }
}

impl Default for MouseController {
    fn default() -> Self {
        Self::new().expect("Failed to create default MouseController")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_mouse_controller_creation() {
        let controller = MouseController::new();
        assert!(controller.is_ok());
    }

    #[test]
    fn test_speed_to_pixels() {
        assert_eq!(MouseController::speed_to_pixels(MovementSpeed::Slow), 5);
        assert_eq!(MouseController::speed_to_pixels(MovementSpeed::Normal), 15);
        assert_eq!(MouseController::speed_to_pixels(MovementSpeed::Fast), 30);
    }

    #[test]
    fn test_position_validation() {
        let controller = MouseController::new().unwrap();
        
        // Test normal position
        let pos = Position::new(100, 100);
        let validated = controller.validate_position(pos);
        assert!(validated.is_ok());
        
        // Test position outside bounds (should be clamped)
        let pos = Position::new(-100, -100);
        let validated = controller.validate_position(pos).unwrap();
        assert!(validated.x >= 0);
        assert!(validated.y >= 0);
    }

    #[test]
    fn test_movement_speed_setting() {
        let mut controller = MouseController::new().unwrap();
        
        controller.set_movement_speed(MovementSpeed::Fast);
        assert_eq!(controller.get_movement_speed(), MovementSpeed::Fast);
        
        controller.set_movement_speed(MovementSpeed::Slow);
        assert_eq!(controller.get_movement_speed(), MovementSpeed::Slow);
    }

    #[test]
    fn test_screen_bounds_detection() {
        let controller = MouseController::new().unwrap();
        let screens = controller.get_screen_bounds().unwrap();
        
        assert!(!screens.is_empty());
        assert!(screens.iter().any(|s| s.is_primary));
    }

    #[test]
    fn test_instant_movement_performance() {
        let mut controller = MouseController::new().unwrap();
        let _start_pos = Position::new(100, 100);
        let end_pos = Position::new(200, 200);
        
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Instant);
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        // Instant movement should be reasonably fast (50ms max due to system calls)
        assert!(duration.as_millis() < 50);
    }

    #[test]
    fn test_linear_animation_performance() {
        let mut controller = MouseController::new().unwrap();
        controller.set_movement_speed(MovementSpeed::Fast);
        
        let _start_pos = Position::new(100, 100);
        let end_pos = Position::new(300, 300);
        
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Linear);
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        // Fast linear animation should complete within reasonable time (200ms max)
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_smooth_animation_performance() {
        let mut controller = MouseController::new().unwrap();
        controller.set_movement_speed(MovementSpeed::Fast);
        
        let _start_pos = Position::new(100, 100);
        let end_pos = Position::new(400, 400);
        
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Smooth);
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        // Smooth animation should complete within reasonable time (200ms max)
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_bounce_animation_performance() {
        let mut controller = MouseController::new().unwrap();
        controller.set_movement_speed(MovementSpeed::Fast);
        
        let _start_pos = Position::new(100, 100);
        let end_pos = Position::new(500, 500);
        
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Bounce);
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        // Bounce animation should complete within reasonable time (200ms max)
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_animation_with_different_speeds() {
        let mut controller = MouseController::new().unwrap();
        let start_pos = Position::new(100, 100);
        let end_pos = Position::new(200, 200);
        
        // Test slow speed
        controller.set_movement_speed(MovementSpeed::Slow);
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Smooth);
        let slow_duration = start_time.elapsed();
        assert!(result.is_ok());
        
        // Reset position
        controller.move_to(start_pos, AnimationType::Instant).unwrap();
        
        // Test fast speed
        controller.set_movement_speed(MovementSpeed::Fast);
        let start_time = Instant::now();
        let result = controller.move_to(end_pos, AnimationType::Smooth);
        let fast_duration = start_time.elapsed();
        assert!(result.is_ok());
        
        // Fast should be quicker than slow
        assert!(fast_duration < slow_duration);
    }

    #[test]
    fn test_relative_movement_performance() {
        let mut controller = MouseController::new().unwrap();
        controller.set_movement_speed(MovementSpeed::Fast);
        
        let start_time = Instant::now();
        let result = controller.move_relative(50, 50);
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        // Relative movement should be reasonably fast (200ms max)
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_screen_switching_performance() {
        let mut controller = MouseController::new().unwrap();
        
        let start_time = Instant::now();
        let _result = controller.move_to_screen(1);
        let duration = start_time.elapsed();
        
        // Screen switching should not take too long regardless of success/failure
        assert!(duration.as_millis() < 200);
    }
}