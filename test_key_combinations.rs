use mouseless_core::{GridManager, GridConfig, ScreenBounds, AnimationType};

fn main() {
    println!("Testing Grid Key Combinations");
    println!("==============================");
    
    // Test different grid sizes
    let test_cases = vec![
        (2, 2, "2x2 Grid"),
        (3, 3, "3x3 Grid"),
        (4, 4, "4x4 Grid"),
        (3, 5, "3x5 Grid"),
        (5, 6, "5x6 Grid"),
    ];
    
    let screen_bounds = ScreenBounds {
        id: 1,
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        is_primary: true,
    };
    
    for (rows, columns, description) in test_cases {
        println!("\n{} ({}x{}):", description, rows, columns);
        println!("{}", "=".repeat(description.len() + 10));
        
        let config = GridConfig {
            rows,
            columns,
            show_labels: true,
            animation_style: AnimationType::Smooth,
            cell_padding: 2,
            border_width: 1,
            opacity: 0.8,
        };
        
        match GridManager::new(config, screen_bounds.clone()) {
            Ok(manager) => {
                let cells = manager.get_cells();
                
                println!("Total cells: {}", cells.len());
                println!("Key combinations:");
                
                for (i, cell) in cells.iter().enumerate() {
                    let row_letter = char::from(b'A' + cell.row as u8);
                    println!(
                        "  Cell {}: {} -> Row {}, Col {} -> Position ({}, {})",
                        i + 1,
                        cell.key_combination.to_uppercase(),
                        row_letter,
                        cell.column + 1,
                        cell.center_position.x,
                        cell.center_position.y
                    );
                }
                
                // Test key lookup
                if let Some(first_cell) = cells.first() {
                    let key_combo = &first_cell.key_combination;
                    if let Some(found_cell) = manager.get_cell_by_keys(key_combo) {
                        println!("\n✓ Key lookup test passed: '{}' -> ({}, {})", 
                                key_combo.to_uppercase(), 
                                found_cell.center_position.x, 
                                found_cell.center_position.y);
                    } else {
                        println!("\n✗ Key lookup test failed for '{}'", key_combo);
                    }
                }
                
                // Test position lookup
                let center_pos = screen_bounds.center();
                if let Some(cell_at_center) = manager.find_cell_at_position(center_pos) {
                    println!("✓ Position lookup test passed: ({}, {}) -> Cell '{}'", 
                            center_pos.x, center_pos.y, 
                            cell_at_center.key_combination.to_uppercase());
                } else {
                    println!("✗ Position lookup test failed for center position");
                }
            }
            Err(e) => {
                println!("✗ Failed to create grid manager: {}", e);
            }
        }
    }
    
    println!("\n\nKey Generation Pattern:");
    println!("======================");
    println!("First keys (home row): a, s, d, f, g, h, j, k, l");
    println!("Second keys (top row): q, w, e, r, t, y, u, i, o, p");
    println!("Combinations: aq, aw, ae, ar, at, ay, au, ai, ao, ap, sq, sw, ...");
    println!("\nThis ensures ergonomic key combinations that are easy to type!");
}