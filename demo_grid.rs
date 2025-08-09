use mouseless_core::{GridManager, GridConfig, ScreenBounds, AnimationType};

fn main() {
    println!("🎯 Mouseless 网格模式演示");
    println!("========================");
    
    // 创建一个 3x3 网格配置
    let config = GridConfig {
        rows: 3,
        columns: 3,
        show_labels: true,
        animation_style: AnimationType::Smooth,
        cell_padding: 2,
        border_width: 1,
        opacity: 0.8,
    };
    
    // 模拟屏幕边界（1920x1080）
    let screen_bounds = ScreenBounds {
        id: 1,
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        is_primary: true,
    };
    
    // 创建网格管理器
    match GridManager::new(config, screen_bounds) {
        Ok(grid_manager) => {
            println!("\n✅ 网格创建成功！");
            println!("网格大小: 3x3 (9个单元格)");
            println!("屏幕尺寸: 1920x1080");
            
            // 显示所有网格单元格和对应的键位组合
            println!("\n📋 网格布局:");
            println!("┌─────────┬─────────┬─────────┐");
            
            let cells = grid_manager.get_cells();
            for row in 0..3 {
                print!("│");
                for col in 0..3 {
                    let cell_index = row * 3 + col;
                    if let Some(cell) = cells.get(cell_index) {
                        print!("   {}   │", cell.key_combination.to_uppercase());
                    }
                }
                println!();
                
                if row < 2 {
                    println!("├─────────┼─────────┼─────────┤");
                } else {
                    println!("└─────────┴─────────┴─────────┘");
                }
            }
            
            // 演示键位查找功能
            println!("\n🎯 键位演示:");
            let demo_keys = ["aq", "aw", "ae", "sq", "sw", "se", "dq", "dw", "de"];
            
            for (i, key_combo) in demo_keys.iter().enumerate() {
                if let Some(cell) = grid_manager.get_cell_by_keys(key_combo) {
                    let row_name = match cell.row {
                        0 => "上",
                        1 => "中",
                        2 => "下",
                        _ => "?"
                    };
                    let col_name = match cell.column {
                        0 => "左",
                        1 => "中",
                        2 => "右",
                        _ => "?"
                    };
                    
                    println!("  {} → {}{}角 ({}, {})", 
                            key_combo.to_uppercase(),
                            row_name, col_name,
                            cell.center_position.x, 
                            cell.center_position.y);
                }
            }
            
            // 演示位置查找功能
            println!("\n🔍 位置查找演示:");
            let test_positions = [
                (320, 180),   // 左上角区域
                (960, 540),   // 屏幕中心
                (1600, 900),  // 右下角区域
            ];
            
            for (x, y) in test_positions {
                let pos = mouseless_core::Position::new(x, y);
                if let Some(cell) = grid_manager.find_cell_at_position(pos) {
                    println!("  位置 ({}, {}) → 网格单元格 {}", 
                            x, y, cell.key_combination.to_uppercase());
                } else {
                    println!("  位置 ({}, {}) → 未找到对应网格", x, y);
                }
            }
            
            println!("\n🚀 使用方法:");
            println!("1. 运行应用: cargo run --bin mouseless-app");
            println!("2. 按 'g' 键激活网格模式");
            println!("3. 按上表中的双键组合移动鼠标");
            println!("4. 按空格键或ESC退出网格模式");
            
            println!("\n💡 提示:");
            println!("- 第一个键使用主行键: a s d f g h j k l");
            println!("- 第二个键使用数字行键: q w e r t y u i o p");
            println!("- 键位组合有1秒超时时间");
            println!("- 支持自定义网格大小和外观");
        }
        Err(e) => {
            println!("❌ 网格创建失败: {}", e);
        }
    }
}