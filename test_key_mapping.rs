use mouseless_core::traits::KeyBindings;

fn main() {
    println!("🔍 调试CapsLock + j/l/i/k 鼠标移动问题");
    println!("=====================================");
    
    let bindings = KeyBindings::default();
    
    println!("📋 当前默认键位绑定:");
    println!("  向上移动 (i): '{}'", bindings.move_up);
    println!("  向下移动 (k): '{}'", bindings.move_down);
    println!("  向左移动 (j): '{}'", bindings.move_left);
    println!("  向右移动 (l): '{}'", bindings.move_right);
    println!("  左键点击 (n): '{}'", bindings.left_click);
    println!("  右键点击 (m): '{}'", bindings.right_click);
    println!("  退出键 (space): '{}'", bindings.exit_key);
    
    println!("\n🎯 问题分析:");
    println!("1. 默认键位绑定看起来是正确的 (i/k/j/l)");
    println!("2. 需要检查以下几个方面:");
    println!("   - CapsLock 激活是否正常工作");
    println!("   - 键盘事件监听是否正确");
    println!("   - 鼠标移动服务是否正常");
    println!("   - macOS 辅助功能权限是否已授予");
    
    println!("\n🔧 建议的调试步骤:");
    println!("1. 检查 macOS 系统偏好设置 > 安全性与隐私 > 辅助功能");
    println!("2. 确保应用已被授予辅助功能权限");
    println!("3. 尝试双击 CapsLock 激活模式");
    println!("4. 查看控制台日志输出");
    
    // 测试键位映射
    println!("\n🧪 键位映射测试:");
    let test_keys = ['i', 'k', 'j', 'l'];
    for key in test_keys {
        let action = match key {
            k if k == bindings.move_up => "向上移动",
            k if k == bindings.move_down => "向下移动", 
            k if k == bindings.move_left => "向左移动",
            k if k == bindings.move_right => "向右移动",
            _ => "未知动作"
        };
        println!("  按键 '{}' -> {}", key, action);
    }
}