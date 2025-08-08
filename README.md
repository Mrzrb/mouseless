# Mouseless

A Rust-based keyboard mouse control tool for macOS that enables users to control mouse operations entirely through keyboard inputs, helping to reduce wrist strain and improve productivity.

## Project Structure

This project uses a Cargo workspace with two main crates:

### `mouseless-core`
The core library containing:
- **Error handling**: Comprehensive error types using `thiserror`
- **Data models**: Core data structures for positions, screens, configurations, etc.
- **Traits**: Interfaces for input processing, mouse operations, mode control, UI rendering, and prediction
- **Logging**: Structured logging setup with `tracing`

### `mouseless-app`
The main application that:
- Integrates with Tauri for UI overlays
- Implements the core traits from `mouseless-core`
- Provides the main entry point and application lifecycle management

## Core Traits

The application is built around five main traits:

1. **`InputProcessor`**: Handles global keyboard input and hotkey registration
2. **`MouseOperations`**: Controls mouse movement, clicking, and scrolling
3. **`ModeController`**: Manages different interaction modes (Basic, Grid, Area, Prediction)
4. **`UIRenderer`**: Renders overlays and visual feedback
5. **`PredictionModel`**: Provides intelligent target prediction

## Features

- **Multiple interaction modes**:
  - Basic: Direct cursor movement with I/K/J/L keys
  - Grid: Quick positioning using a visual grid overlay
  - Area: 9-area screen division for rapid navigation
  - Prediction: AI-powered target prediction

- **Smooth animations**: Configurable cursor movement with easing
- **Multi-monitor support**: Seamless operation across multiple displays
- **Customizable key bindings**: Adapt to personal workflow preferences
- **Modern UI**: Glassmorphism effects and smooth transitions
- **Performance optimized**: Sub-10ms response times, minimal resource usage

## Development

### Prerequisites

- Rust 1.70+
- macOS 10.15+
- Xcode Command Line Tools

### Building

```bash
# Build the entire workspace
cargo build

# Build in release mode
cargo build --release

# Run the application
cargo run -p mouseless-app

# Run tests
cargo test
```

### Logging

The application uses structured logging with `tracing`. Set environment variables to control log levels:

```bash
# Enable debug logging for all components
RUST_LOG=mouseless=debug cargo run -p mouseless-app

# Enable JSON structured logging
MOUSELESS_JSON_LOGS=1 cargo run -p mouseless-app
```

## Requirements

This implementation addresses the following key requirements:
- **7.5**: macOS compatibility and system integration
- **8.5**: Performance optimization with minimal resource usage

## License

[License information to be added]