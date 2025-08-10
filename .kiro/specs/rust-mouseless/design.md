# 设计文档

## 概述

本文档概述了基于 Rust 的键盘鼠标控制工具的技术设计，该工具提供多种交互模式，包括基本光标移动、基于网格的定位和智能预测。该应用程序将专门为 macOS 构建，利用 Rust 的性能和安全特性，同时提供现代化的动画用户界面。

系统采用模块化架构，在输入处理、鼠标控制、UI 渲染和机器学习组件之间有清晰的分离。设计强调性能、用户体验和可扩展性。所有配置通过 `~/.mouseless.toml` 文件管理，无需 GUI 设置界面。

## 架构

### 高级架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用程序层                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   UI 管理器     │  │   模式管理器    │  │  配置管理器     │  │
│  └─────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        核心服务                                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   输入处理器    │  │   鼠标控制器    │  │   预测 AI       │  │
│  └─────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        平台层                                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   全局热键      │  │   鼠标事件      │  │   辅助功能      │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 技术栈

基于 Context7 研究，以下技术栈已经过验证：

- **核心框架**: 原生 Rust 应用程序，使用 Tauri 进行 UI 覆盖层
  - Tauri 提供出色的桌面应用程序开发和 Web 前端
  - 支持覆盖层窗口、事件处理和系统集成
  - 小的二进制文件大小和良好的性能特征
- **鼠标控制**: `enigo` crate 用于跨平台鼠标模拟
  - 经过验证的鼠标移动、点击和滚动库
  - 通过原生 API 支持 macOS
  - 良好的性能，响应时间低于 10ms
- **全局热键**: `global-hotkey` crate 用于系统级按键捕获
  - 处理整个系统的全局键盘事件
  - 与 macOS 辅助功能权限配合良好
- **UI 渲染**: Tauri 配合 HTML/CSS/JavaScript 前端用于覆盖层
  - 支持现代 Web 技术实现美丽的动画
  - 支持玻璃态效果和流畅过渡
  - 易于实现网格覆盖层和视觉反馈
- **机器学习**: 最初使用简单的基于启发式的预测，后续可能使用 ML
  - 从基于规则的预测开始（UI 元素检测、使用模式）
  - 考虑使用 `candle-core` 或 `tch` 进行未来的 ML 实现
- **配置管理**: `serde` 配合 TOML 支持，默认使用 `~/.mouseless.toml`
- **日志记录**: `tracing` 用于结构化日志
- **异步运行时**: `tokio` 用于异步操作

## 组件和接口

### 1. 输入处理器组件

**职责**: 捕获和处理全局键盘输入事件

```rust
pub struct InputHandler {
    hotkey_manager: GlobalHotKeyManager,
    event_receiver: GlobalHotKeyEventReceiver,
    key_bindings: KeyBindings,
    current_mode: Arc<Mutex<InputMode>>,
    config_watcher: ConfigWatcher,
}

pub trait InputProcessor {
    async fn process_key_event(&self, event: KeyEvent) -> Result<Action>;
    fn register_hotkey(&mut self, hotkey: HotKey) -> Result<()>;
    fn update_bindings(&mut self, bindings: KeyBindings) -> Result<()>;
    fn reload_config(&mut self) -> Result<()>;
}
```

**主要特性**:
- 全局热键注册和管理
- 可配置的按键绑定系统
- 模式感知的输入处理
- 事件过滤和验证
- 配置文件热重载支持

### 2. 鼠标控制器组件

**职责**: 执行鼠标移动、点击和滚动操作

```rust
pub struct MouseController {
    enigo: Enigo,
    current_position: Arc<Mutex<Position>>,
    movement_config: MovementConfig,
    screen_info: MultiScreenInfo,
}

pub trait MouseOperations {
    async fn move_to(&mut self, position: Position, animation: AnimationType) -> Result<()>;
    async fn click(&mut self, button: MouseButton) -> Result<()>;
    async fn scroll(&mut self, direction: ScrollDirection, amount: i32) -> Result<()>;
    fn get_current_position(&self) -> Position;
    fn get_all_screen_bounds(&self) -> Vec<ScreenBounds>;
    fn update_movement_config(&mut self, config: MovementConfig);
}
```

**主要特性**:
- 流畅的动画光标移动
- 多显示器支持（网格跨所有屏幕）
- 可配置的移动速度
- 点击和滚动操作
- 配置驱动的行为调整

### 3. 模式管理器组件

**职责**: 管理不同的交互模式及其转换

```rust
pub enum InteractionMode {
    Basic(BasicMode),
    Grid(GridMode),
    Prediction(PredictionMode),
}

pub struct ModeManager {
    current_mode: Arc<Mutex<InteractionMode>>,
    mode_history: Vec<InteractionMode>,
    ui_manager: Arc<UIManager>,
    config: ModeConfig,
}

pub trait ModeController {
    async fn activate_mode(&mut self, mode: InteractionMode) -> Result<()>;
    async fn deactivate_current_mode(&mut self) -> Result<()>;
    fn get_current_mode(&self) -> InteractionMode;
    async fn handle_input(&self, input: KeyInput) -> Result<Action>;
    fn update_mode_config(&mut self, config: ModeConfig);
}
```

### 4. UI 管理器组件

**职责**: 渲染覆盖层、动画和视觉反馈

```rust
pub struct UIManager {
    tauri_app: Arc<tauri::App>,
    overlay_windows: HashMap<String, WebviewWindow>,
    animation_engine: AnimationEngine,
    theme_manager: ThemeManager,
}

pub trait UIRenderer {
    async fn show_grid_overlay(&self, grid_config: GridConfig) -> Result<()>;
    async fn show_prediction_targets(&self, targets: Vec<PredictionTarget>) -> Result<()>;
    async fn animate_cursor_movement(&self, from: Position, to: Position) -> Result<()>;
    async fn hide_all_overlays(&self) -> Result<()>;
}
```

### 5. 预测 AI 组件

**职责**: 学习用户模式并预测可能的点击目标

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

## 数据模型

### 核心数据结构

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

### 配置模型

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

## 错误处理

### 错误类型

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

### 错误恢复策略

1. **优雅降级**: 如果高级功能失败，回退到基本功能
2. **用户通知**: 显示清晰的错误消息和可操作的解决方案
3. **自动恢复**: 尝试从瞬态错误中自动恢复
4. **安全模式**: 在关键组件失败时提供最小功能模式

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

## 实现阶段

### 阶段 1: 核心基础设施 (第 1-2 周)
- 使用 Cargo workspace 设置 Rust 项目结构
- 使用 global-hotkey 实现基本输入处理
- 使用 enigo 集成创建鼠标控制器
- 使用 serde 建立配置系统，支持 `~/.mouseless.toml`
- 设置日志记录和错误处理框架

### 阶段 2: 基本功能 (第 3-4 周)
- 实现基本光标移动 (I/K/J/L 键)
- 添加点击操作 (N/M 键用于左/右键点击)
- 实现滚动功能 (U/O/Y/P 键)
- 创建激活/停用系统
- 添加多显示器支持

### 阶段 3: 高级模式 (第 5-7 周)
- 实现带可配置覆盖层的网格模式
- 添加模式切换和管理系统
- 实现视觉反馈和基本动画
- 创建基于配置文件的自定义系统

### 阶段 4: UI 和动画 (第 8-9 周)
- 集成 Tauri 进行覆盖层渲染
- 实现流畅的光标移动动画
- 创建美丽的网格覆盖层
- 添加玻璃态和现代视觉效果
- 实现主题系统和深色模式支持

### 阶段 5: 智能预测 (第 10-12 周)
- 使用 macOS 辅助功能 API 实现屏幕内容分析
- 创建本地存储的使用模式跟踪系统
- 构建基于启发式的预测模型（按钮检测、表单字段、常见 UI 模式）
- 添加带置信度指示器的预测目标可视化
- 实现学习和反馈机制以改进模式

### 阶段 6: 完善和优化 (第 13-14 周)
- 性能优化和内存管理
- macOS 特定集成和权限处理
- 全面测试和错误修复
- 文档和用户指南
- 打包和分发设置