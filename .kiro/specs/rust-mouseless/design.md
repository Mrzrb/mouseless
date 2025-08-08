# Design Document

## Overview

This document outlines the technical design for a Rust-based keyboard mouse control tool that provides multiple interaction modes including basic cursor movement, grid-based positioning, area-based navigation, and intelligent prediction. The application will be built specifically for macOS, leveraging Rust's performance and safety features while providing a modern, animated user interface.

The system follows a modular architecture with clear separation between input handling, mouse control, UI rendering, and machine learning components. The design emphasizes performance, user experience, and extensibility.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   UI Manager    │  │  Mode Manager   │  │ Config Manager  │  │
│  └─────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        Core Services                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Input Handler   │  │ Mouse Controller│  │ Prediction AI   │  │
│  └─────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        Platform Layer                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Global Hotkeys  │  │  Mouse Events   │  │ Accessibility   │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

Based on Context7 research, the following technology stack has been validated:

- **Core Framework**: Native Rust application with Tauri for UI overlay
  - Tauri provides excellent desktop app development with web frontend
  - Supports overlay windows, event handling, and system integration
  - Small binary size and good performance characteristics
- **Mouse Control**: `enigo` crate for cross-platform mouse simulation
  - Proven library for mouse movement, clicking, and scrolling
  - Supports macOS through native APIs
  - Good performance with sub-10ms response times
- **Global Hotkeys**: `global-hotkey` crate for system-wide key capture
  - Handles global keyboard events across the system
  - Works well with macOS accessibility permissions
- **UI Rendering**: Tauri with HTML/CSS/JavaScript frontend for overlays
  - Enables modern web technologies for beautiful animations
  - Supports glassmorphism effects and smooth transitions
  - Easy to implement grid overlays and visual feedback
- **Machine Learning**: Simple heuristic-based prediction initially, with potential for ML later
  - Start with rule-based prediction (UI element detection, usage patterns)
  - Consider `candle-core` or `tch` for future ML implementation
- **Configuration**: `serde` with JSON/TOML support
- **Logging**: `tracing` for structured logging
- **Async Runtime**: `tokio` for asynchronous operations

## Components and Interfaces

### 1. Input Handler Component

**Responsibility**: Capture and process global keyboard input events

```rust
pub struct InputHandler {
    hotkey_manager: GlobalHotKeyManager,
    event_receiver: GlobalHotKeyEventReceiver,
    key_bindings: KeyBindings,
    current_mode: Arc<Mutex<InputMode>>,
}

pub trait InputProcessor {
    async fn process_key_event(&self, event: KeyEvent) -> Result<Action>;
    fn register_hotkey(&mut self, hotkey: HotKey) -> Result<()>;
    fn update_bindings(&mut self, bindings: KeyBindings) -> Result<()>;
}
```

**Key Features**:
- Global hotkey registration and management
- Configurable key binding system
- Mode-aware input processing
- Event filtering and validation

### 2. Mouse Controller Component

**Responsibility**: Execute mouse movements, clicks, and scrolling operations

```rust
pub struct MouseController {
    enigo: Enigo,
    current_position: Arc<Mutex<Position>>,
    movement_speed: MovementSpeed,
    screen_info: ScreenInfo,
}

pub trait MouseOperations {
    async fn move_to(&mut self, position: Position, animation: AnimationType) -> Result<()>;
    async fn click(&mut self, button: MouseButton) -> Result<()>;
    async fn scroll(&mut self, direction: ScrollDirection, amount: i32) -> Result<()>;
    fn get_current_position(&self) -> Position;
    fn get_screen_bounds(&self) -> Vec<ScreenBounds>;
}
```

**Key Features**:
- Smooth animated cursor movement
- Multi-monitor support
- Configurable movement speeds
- Click and scroll operations

### 3. Mode Manager Component

**Responsibility**: Manage different interaction modes and their transitions

```rust
pub enum InteractionMode {
    Basic(BasicMode),
    Grid(GridMode),
    Area(AreaMode),
    Prediction(PredictionMode),
}

pub struct ModeManager {
    current_mode: Arc<Mutex<InteractionMode>>,
    mode_history: Vec<InteractionMode>,
    ui_manager: Arc<UIManager>,
}

pub trait ModeController {
    async fn activate_mode(&mut self, mode: InteractionMode) -> Result<()>;
    async fn deactivate_current_mode(&mut self) -> Result<()>;
    fn get_current_mode(&self) -> InteractionMode;
    async fn handle_input(&self, input: KeyInput) -> Result<Action>;
}
```

### 4. UI Manager Component

**Responsibility**: Render overlays, animations, and visual feedback

```rust
pub struct UIManager {
    tauri_app: Arc<tauri::App>,
    overlay_windows: HashMap<String, WebviewWindow>,
    animation_engine: AnimationEngine,
    theme_manager: ThemeManager,
}

pub trait UIRenderer {
    async fn show_grid_overlay(&self, grid_config: GridConfig) -> Result<()>;
    async fn show_area_overlay(&self, areas: Vec<Area>) -> Result<()>;
    async fn show_prediction_targets(&self, targets: Vec<PredictionTarget>) -> Result<()>;
    async fn animate_cursor_movement(&self, from: Position, to: Position) -> Result<()>;
    async fn hide_all_overlays(&self) -> Result<()>;
}
```

### 5. Prediction AI Component

**Responsibility**: Learn user patterns and predict likely click targets

```rust
pub struct PredictionEngine {
    model: Box<dyn PredictionModel>,
    usage_tracker: UsageTracker,
    screen_analyzer: ScreenAnalyzer,
    confidence_threshold: f32,
}

pub trait PredictionModel {
    fn predict_targets(&self, context: ScreenContext) -> Vec<PredictionTarget>;
    fn update_model(&mut self, feedback: UserFeedback) -> Result<()>;
    fn get_confidence(&self, target: &PredictionTarget) -> f32;
}
```

## Data Models

### Core Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub screen_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ScreenBounds {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct GridConfig {
    pub rows: u32,
    pub columns: u32,
    pub cell_size: (u32, u32),
    pub show_labels: bool,
    pub animation_style: AnimationStyle,
}

#[derive(Debug, Clone)]
pub struct PredictionTarget {
    pub position: Position,
    pub confidence: f32,
    pub target_type: TargetType,
    pub shortcut_key: char,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TargetType {
    Button,
    Link,
    TextField,
    MenuItem,
    Icon,
    Custom(String),
}
```

### Configuration Models

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub activation: ActivationConfig,
    pub movement: MovementConfig,
    pub ui: UIConfig,
    pub prediction: PredictionConfig,
    pub keybindings: KeyBindings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivationConfig {
    pub trigger_key: Key,
    pub modifier_keys: Vec<Modifier>,
    pub double_click_required: bool,
    pub activation_timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: Theme,
    pub animation_speed: f32,
    pub overlay_opacity: f32,
    pub show_cursor_trail: bool,
    pub glassmorphism_enabled: bool,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MouselessError {
    #[error("Input handling error: {0}")]
    InputError(#[from] InputError),
    
    #[error("Mouse control error: {0}")]
    MouseError(#[from] enigo::InputError),
    
    #[error("UI rendering error: {0}")]
    UIError(#[from] tauri::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("Permission denied: {message}")]
    PermissionError { message: String },
    
    #[error("Prediction model error: {0}")]
    PredictionError(String),
}
```

### Error Recovery Strategy

1. **Graceful Degradation**: If advanced features fail, fall back to basic functionality
2. **User Notification**: Display clear error messages with actionable solutions
3. **Automatic Recovery**: Attempt to recover from transient errors automatically
4. **Safe Mode**: Provide a minimal functionality mode when critical components fail

## Testing Strategy

### Unit Testing

- **Component Isolation**: Test each component independently with mocked dependencies
- **Input Validation**: Verify proper handling of edge cases and invalid inputs
- **State Management**: Test state transitions and concurrent access patterns
- **Configuration Loading**: Validate configuration parsing and validation logic

### Integration Testing

- **End-to-End Workflows**: Test complete user interaction flows
- **Platform Integration**: Verify macOS-specific functionality and permissions
- **Performance Testing**: Measure response times and resource usage
- **UI Testing**: Automated testing of overlay rendering and animations

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_mouse_movement_animation() {
        let mut mock_controller = MockMouseController::new();
        mock_controller
            .expect_move_to()
            .with(eq(Position::new(100, 100)), eq(AnimationType::Smooth))
            .times(1)
            .returning(|_, _| Ok(()));
            
        let result = mock_controller.move_to(
            Position::new(100, 100), 
            AnimationType::Smooth
        ).await;
        
        assert!(result.is_ok());
    }
}
```

### Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_prediction_engine(c: &mut Criterion) {
        c.bench_function("predict_targets", |b| {
            b.iter(|| {
                let engine = PredictionEngine::new();
                let context = ScreenContext::mock();
                black_box(engine.predict_targets(context))
            })
        });
    }
    
    criterion_group!(benches, benchmark_prediction_engine);
    criterion_main!(benches);
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
- Set up Rust project structure with Cargo workspace
- Implement basic input handling with global-hotkey
- Create mouse controller with enigo integration
- Establish configuration system with serde
- Set up logging and error handling framework

### Phase 2: Basic Functionality (Weeks 3-4)
- Implement basic cursor movement (I/K/J/L keys)
- Add click operations (N/M keys for left/right click)
- Implement scrolling functionality (U/O/Y/P keys)
- Create activation/deactivation system
- Add multi-monitor support

### Phase 3: Advanced Modes (Weeks 5-7)
- Implement grid mode with configurable overlay
- Create area mode with 9-region division
- Add mode switching and management system
- Implement visual feedback and basic animations
- Create settings and customization interface

### Phase 4: UI and Animations (Weeks 8-9)
- Integrate Tauri for overlay rendering
- Implement smooth cursor movement animations
- Create beautiful grid and area overlays
- Add glassmorphism and modern visual effects
- Implement theme system and dark mode support

### Phase 5: Intelligent Prediction (Weeks 10-12)
- Implement screen content analysis using macOS Accessibility API
- Create usage pattern tracking system with local storage
- Build heuristic-based prediction model (button detection, form fields, common UI patterns)
- Add prediction target visualization with confidence indicators
- Implement learning and feedback mechanisms for pattern improvement

### Phase 6: Polish and Optimization (Weeks 13-14)
- Performance optimization and memory management
- macOS-specific integration and permissions handling
- Comprehensive testing and bug fixes
- Documentation and user guides
- Packaging and distribution setup