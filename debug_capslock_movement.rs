use mouseless_core::{
    init, ActivationConfig, ActivationKey, InputHandler, KeyBindings, ModeManager, MouseService,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 初始化核心库
    if let Err(e) = init() {
        eprintln!("Failed to initialize core library: {}", e);
        return Err(e.into());
    }

    info!("🚀 Starting CapsLock movement debug test...");

    // 创建输入处理器
    let mut input_handler = InputHandler::new()?;
    
    // 配置激活设置 - 使用CapsLock双击激活
    let activation_config = ActivationConfig {
        trigger_key: ActivationKey::CapsLock,
        modifier_keys: vec![],
        double_click_required: true,
        double_click_timeout_ms: 300,
        activation_timeout_ms: 5000,
    };
    
    input_handler.update_activation_config(activation_config)?;
    input_handler.register_activation_hotkey()?;
    
    // 设置动作通道
    let action_receiver = input_handler.setup_action_channel();
    
    // 启动输入事件循环
    let input_handler_arc = Arc::new(Mutex::new(input_handler));
    let input_handler_for_spawn = Arc::clone(&input_handler_arc);
    
    tokio::spawn(async move {
        let mut handler = input_handler_for_spawn.lock().await;
        if let Err(e) = handler.start_event_loop().await {
            error!("Failed to start input event loop: {}", e);
        }
    });

    // 创建模式管理器
    let key_bindings = KeyBindings::default();
    info!("📋 Key bindings configured:");
    info!("  Move up (i): '{}'", key_bindings.move_up);
    info!("  Move down (k): '{}'", key_bindings.move_down);
    info!("  Move left (j): '{}'", key_bindings.move_left);
    info!("  Move right (l): '{}'", key_bindings.move_right);
    info!("  Exit key (space): '{}'", key_bindings.exit_key);
    
    let mode_manager = Arc::new(Mutex::new(ModeManager::new(key_bindings)));
    
    // 创建鼠标服务
    let mouse_service = MouseService::new();
    
    info!("🎯 System ready! Instructions:");
    info!("1. Double-click CapsLock to activate mouseless mode");
    info!("2. Use i/k/j/l keys to move mouse (up/down/left/right)");
    info!("3. Press Space to exit mode");
    info!("4. Press Ctrl+C to quit this debug program");
    
    // 处理动作
    let mut action_receiver = action_receiver;
    while let Some(action) = action_receiver.recv().await {
        info!("📨 Received action: {:?}", action);
        
        match action {
            mouseless_core::Action::MoveCursor(position, _animation_type) => {
                info!("🖱️ Moving mouse by ({}, {})", position.x, position.y);
                if let Err(e) = mouse_service.move_to_relative(position.x, position.y).await {
                    error!("❌ Failed to move cursor: {}", e);
                } else {
                    info!("✅ Mouse moved successfully");
                }
            }
            mouseless_core::Action::Click(button) => {
                info!("🖱️ Clicking mouse button: {:?}", button);
                if let Err(e) = mouse_service.click(button).await {
                    error!("❌ Failed to click: {}", e);
                } else {
                    info!("✅ Mouse clicked successfully");
                }
            }
            mouseless_core::Action::ActivateMode(mode) => {
                info!("🎯 Activating mode: {:?}", mode);
                let mut manager = mode_manager.lock().await;
                if let Err(e) = manager.activate_mode(mode).await {
                    error!("❌ Failed to activate mode: {}", e);
                } else {
                    info!("✅ Mode activated successfully");
                }
            }
            mouseless_core::Action::Exit => {
                info!("🚪 Exiting mode");
                let mut manager = mode_manager.lock().await;
                if let Err(e) = manager.deactivate_current_mode().await {
                    error!("❌ Failed to deactivate mode: {}", e);
                } else {
                    info!("✅ Mode deactivated successfully");
                }
            }
            mouseless_core::Action::ToggleSpeed => {
                info!("⚡ Toggling speed");
                let manager = mode_manager.lock().await;
                manager.toggle_speed();
            }
            _ => {
                info!("ℹ️ Other action received: {:?}", action);
            }
        }
    }
    
    Ok(())
}