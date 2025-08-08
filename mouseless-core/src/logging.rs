use tracing::info;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initialize the logging system with structured output
pub fn init_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create environment filter with default level
    let filter_str = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "mouseless=debug,info".to_string());

    // Create console layer with pretty formatting
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(EnvFilter::new(&filter_str));

    // Create JSON layer for structured logging (optional, can be enabled via env var)
    let json_layer = if std::env::var("MOUSELESS_JSON_LOGS").is_ok() {
        Some(
            fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_filter(EnvFilter::new(&filter_str)),
        )
    } else {
        None
    };

    // Initialize the subscriber
    let registry = tracing_subscriber::registry().with(console_layer);

    if let Some(json_layer) = json_layer {
        registry.with(json_layer).init();
    } else {
        registry.init();
    }

    info!("Logging system initialized");
    Ok(())
}

/// Log levels for different components
pub mod levels {
    use tracing::Level;

    pub const INPUT: Level = Level::DEBUG;
    pub const MOUSE: Level = Level::DEBUG;
    pub const UI: Level = Level::INFO;
    pub const PREDICTION: Level = Level::INFO;
    pub const CONFIG: Level = Level::WARN;
    pub const PERFORMANCE: Level = Level::TRACE;
}

/// Structured logging macros for different components
#[macro_export]
macro_rules! log_input {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::input", $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_mouse {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::mouse", $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_ui {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::ui", $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_prediction {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::prediction", $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_config {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::config", $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_performance {
    ($level:expr, $($arg:tt)*) => {
        tracing::event!(target: "mouseless::performance", $level, $($arg)*);
    };
}

/// Performance measurement utilities
pub struct PerformanceTimer {
    start: std::time::Instant,
    operation: String,
}

impl PerformanceTimer {
    pub fn new(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        tracing::debug!(target: "mouseless::performance", "Starting operation: {}", operation);
        Self {
            start: std::time::Instant::now(),
            operation,
        }
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        tracing::debug!(
            target: "mouseless::performance",
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
    }
}

/// Convenience macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $code:block) => {{
        let _timer = $crate::logging::PerformanceTimer::new($operation);
        $code
    }};
}