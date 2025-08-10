use crate::models::{AnimationType, MovementSpeed, Position};
use std::time::Instant;

/// Easing functions for smooth animations
pub struct EasingFunctions;

impl EasingFunctions {
    /// Linear interpolation (no easing)
    pub fn linear(t: f32) -> f32 {
        t
    }

    /// Ease-out cubic for smooth deceleration
    pub fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    /// Ease-in-out cubic for smooth acceleration and deceleration
    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// Ease-out bounce for playful bounce effect
    pub fn ease_out_bounce(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;

        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t = t - 1.5 / D1;
            N1 * t * t + 0.75
        } else if t < 2.5 / D1 {
            let t = t - 2.25 / D1;
            N1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / D1;
            N1 * t * t + 0.984375
        }
    }

    /// Ease-out elastic for elastic bounce effect
    pub fn ease_out_elastic(t: f32) -> f32 {
        const C4: f32 = (2.0 * std::f32::consts::PI) / 3.0;

        if t == 0.0 {
            0.0
        } else if t == 1.0 {
            1.0
        } else {
            2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
        }
    }

    /// Get easing function for animation type
    pub fn get_easing_function(animation_type: AnimationType) -> fn(f32) -> f32 {
        match animation_type {
            AnimationType::Instant => Self::linear,
            AnimationType::Linear => Self::linear,
            AnimationType::Smooth => Self::ease_out_cubic,
            AnimationType::Bounce => Self::ease_out_bounce,
        }
    }
}

/// Animation configuration based on movement speed and animation type
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub duration_ms: u64,
    pub steps: u32,
    pub easing_function: fn(f32) -> f32,
}

impl AnimationConfig {
    /// Create animation config based on speed and type
    pub fn new(speed: MovementSpeed, animation_type: AnimationType) -> Self {
        let (duration_ms, steps) = match speed {
            MovementSpeed::Slow => (300, 30), // Slower, more steps for smoothness
            MovementSpeed::Normal => (150, 20), // Balanced
            MovementSpeed::Fast => (80, 15),  // Faster, fewer steps for responsiveness
        };

        // For instant animation, use minimal duration and steps
        let (duration_ms, steps) = if animation_type == AnimationType::Instant {
            (0, 1)
        } else {
            (duration_ms, steps)
        };

        Self {
            duration_ms,
            steps,
            easing_function: EasingFunctions::get_easing_function(animation_type),
        }
    }

    /// Calculate step delay in milliseconds
    pub fn step_delay_ms(&self) -> u64 {
        if self.steps <= 1 {
            0
        } else {
            self.duration_ms / self.steps as u64
        }
    }
}

/// Animation interpolator for smooth cursor movement
pub struct AnimationInterpolator {
    start_position: Position,
    end_position: Position,
    config: AnimationConfig,
    start_time: Instant,
}

impl AnimationInterpolator {
    /// Create a new animation interpolator
    pub fn new(
        start: Position,
        end: Position,
        speed: MovementSpeed,
        animation_type: AnimationType,
    ) -> Self {
        Self {
            start_position: start,
            end_position: end,
            config: AnimationConfig::new(speed, animation_type),
            start_time: Instant::now(),
        }
    }

    /// Get the next position in the animation sequence
    pub fn next_position(&self, step: u32) -> Option<Position> {
        if step >= self.config.steps {
            return None;
        }

        // For instant animation, return end position immediately
        if self.config.steps == 1 {
            return Some(self.end_position);
        }

        // Calculate progress (0.0 to 1.0)
        let progress = (step + 1) as f32 / self.config.steps as f32;

        // Apply easing function
        let eased_progress = (self.config.easing_function)(progress);

        // Interpolate position
        let dx = self.end_position.x - self.start_position.x;
        let dy = self.end_position.y - self.start_position.y;

        let interpolated_x = self.start_position.x + (dx as f32 * eased_progress) as i32;
        let interpolated_y = self.start_position.y + (dy as f32 * eased_progress) as i32;

        Some(Position {
            x: interpolated_x,
            y: interpolated_y,
            screen_id: self.end_position.screen_id,
        })
    }

    /// Get all animation positions
    pub fn get_animation_sequence(&self) -> Vec<Position> {
        (0..self.config.steps)
            .filter_map(|step| self.next_position(step))
            .collect()
    }

    /// Get animation configuration
    pub fn config(&self) -> &AnimationConfig {
        &self.config
    }

    /// Check if animation should be complete based on elapsed time
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed().as_millis() >= self.config.duration_ms as u128
    }

    /// Get elapsed time since animation start
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// Performance metrics for animation
#[derive(Debug, Clone)]
pub struct AnimationMetrics {
    pub total_duration_ms: u64,
    pub steps_completed: u32,
    pub average_step_time_ms: f64,
    pub max_step_time_ms: u64,
    pub min_step_time_ms: u64,
}

impl AnimationMetrics {
    pub fn new() -> Self {
        Self {
            total_duration_ms: 0,
            steps_completed: 0,
            average_step_time_ms: 0.0,
            max_step_time_ms: 0,
            min_step_time_ms: u64::MAX,
        }
    }

    /// Record a step timing
    pub fn record_step(&mut self, step_duration_ms: u64) {
        self.steps_completed += 1;
        self.total_duration_ms += step_duration_ms;
        self.max_step_time_ms = self.max_step_time_ms.max(step_duration_ms);
        self.min_step_time_ms = self.min_step_time_ms.min(step_duration_ms);
        self.average_step_time_ms = self.total_duration_ms as f64 / self.steps_completed as f64;
    }

    /// Check if performance meets sub-10ms requirement
    pub fn meets_performance_requirement(&self) -> bool {
        //TODO: Make performance requirements configurable via config file
        //TODO: Add performance monitoring and alerting
        //TODO: Implement adaptive performance adjustment
        self.average_step_time_ms < 10.0 && self.max_step_time_ms < 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        assert_eq!(EasingFunctions::linear(0.0), 0.0);
        assert_eq!(EasingFunctions::linear(0.5), 0.5);
        assert_eq!(EasingFunctions::linear(1.0), 1.0);
    }

    #[test]
    fn test_ease_out_cubic() {
        let result = EasingFunctions::ease_out_cubic(0.5);
        assert!(result > 0.5); // Should be accelerated
        assert!(result < 1.0);
    }

    #[test]
    fn test_animation_config_creation() {
        let config = AnimationConfig::new(MovementSpeed::Normal, AnimationType::Smooth);
        assert_eq!(config.duration_ms, 150);
        assert_eq!(config.steps, 20);
    }

    #[test]
    fn test_animation_interpolator() {
        let start = Position::new(0, 0);
        let end = Position::new(100, 100);
        let interpolator =
            AnimationInterpolator::new(start, end, MovementSpeed::Fast, AnimationType::Smooth);

        let first_pos = interpolator.next_position(0).unwrap();
        let last_pos = interpolator
            .next_position(interpolator.config.steps - 1)
            .unwrap();

        // First position should be closer to start
        assert!(first_pos.x < end.x);
        assert!(first_pos.y < end.y);

        // Last position should be the end position
        assert_eq!(last_pos.x, end.x);
        assert_eq!(last_pos.y, end.y);
    }

    #[test]
    fn test_smooth_animation() {
        let start = Position::new(0, 0);
        let end = Position::new(100, 100);
        let interpolator =
            AnimationInterpolator::new(start, end, MovementSpeed::Fast, AnimationType::Smooth);

        assert!(interpolator.config.steps > 1);
        assert!(interpolator.config.duration_ms > 0);

        let pos = interpolator.next_position(0).unwrap();
        // For smooth animation, the first position should be between start and end
        assert!(pos.x > start.x && pos.x <= end.x);
        assert!(pos.y > start.y && pos.y <= end.y);
    }

    #[test]
    fn test_animation_metrics() {
        let mut metrics = AnimationMetrics::new();

        metrics.record_step(5);
        metrics.record_step(8);
        metrics.record_step(3);

        assert_eq!(metrics.steps_completed, 3);
        assert_eq!(metrics.total_duration_ms, 16);
        assert_eq!(metrics.max_step_time_ms, 8);
        assert_eq!(metrics.min_step_time_ms, 3);
        assert!((metrics.average_step_time_ms - 5.33).abs() < 0.1);
        assert!(metrics.meets_performance_requirement());
    }

    #[test]
    fn test_performance_requirement_failure() {
        let mut metrics = AnimationMetrics::new();

        metrics.record_step(15); // Exceeds 10ms requirement

        assert!(!metrics.meets_performance_requirement());
    }
}
