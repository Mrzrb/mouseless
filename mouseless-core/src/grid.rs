use crate::{GridConfig, MouselessError, Position, Result, ScreenBounds};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single grid cell with its position and key combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub row: u32,
    pub column: u32,
    pub bounds: GridCellBounds,
    pub key_combination: String,
    pub center_position: Position,
}

/// Grid cell boundary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCellBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl GridCellBounds {
    pub fn center(&self) -> Position {
        Position::new(
            self.x + (self.width as i32) / 2,
            self.y + (self.height as i32) / 2,
        )
    }

    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.x
            && position.x < self.x + self.width as i32
            && position.y >= self.y
            && position.y < self.y + self.height as i32
    }
}

/// Grid manager handles grid calculations and key combinations
#[derive(Clone)]
pub struct GridManager {
    config: GridConfig,
    cells: Vec<GridCell>,
    key_to_cell: HashMap<String, usize>,
    screen_bounds: ScreenBounds,
}

impl GridManager {
    /// Create a new grid manager with the specified configuration and screen bounds
    pub fn new(config: GridConfig, screen_bounds: ScreenBounds) -> Result<Self> {
        let mut manager = Self {
            config,
            cells: Vec::new(),
            key_to_cell: HashMap::new(),
            screen_bounds,
        };

        manager.calculate_grid()?;
        Ok(manager)
    }

    /// Calculate grid cells and generate key combinations
    fn calculate_grid(&mut self) -> Result<()> {
        self.cells.clear();
        self.key_to_cell.clear();

        let cell_width = self.screen_bounds.width / self.config.columns;
        let cell_height = self.screen_bounds.height / self.config.rows;

        // Generate key combinations for grid cells
        let key_combinations = self.generate_key_combinations()?;

        let mut key_index = 0;

        for row in 0..self.config.rows {
            for col in 0..self.config.columns {
                if key_index >= key_combinations.len() {
                    return Err(MouselessError::SystemError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Not enough key combinations for grid size",
                    )));
                }

                let x = self.screen_bounds.x + (col * cell_width) as i32;
                let y = self.screen_bounds.y + (row * cell_height) as i32;

                let bounds = GridCellBounds {
                    x,
                    y,
                    width: cell_width,
                    height: cell_height,
                };

                let center_position = bounds.center();
                let key_combination = key_combinations[key_index].clone();

                let cell = GridCell {
                    row,
                    column: col,
                    bounds,
                    key_combination: key_combination.clone(),
                    center_position,
                };

                self.key_to_cell.insert(key_combination, self.cells.len());
                self.cells.push(cell);
                key_index += 1;
            }
        }

        Ok(())
    }

    /// Generate two-key combinations for grid cells
    fn generate_key_combinations(&self) -> Result<Vec<String>> {
        let total_cells = (self.config.rows * self.config.columns) as usize;

        // Use common keyboard keys for combinations
        // First set: home row keys for better ergonomics
        let first_keys = ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'];
        let second_keys = ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'];

        let mut combinations = Vec::new();

        // Generate combinations by pairing first and second keys
        for &first in &first_keys {
            for &second in &second_keys {
                if combinations.len() >= total_cells {
                    break;
                }
                combinations.push(format!("{}{}", first, second));
            }
            if combinations.len() >= total_cells {
                break;
            }
        }

        // If we need more combinations, use additional keys
        if combinations.len() < total_cells {
            let additional_first = ['z', 'x', 'c', 'v', 'b', 'n', 'm'];
            let additional_second = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];

            for &first in &additional_first {
                for &second in &additional_second {
                    if combinations.len() >= total_cells {
                        break;
                    }
                    combinations.push(format!("{}{}", first, second));
                }
                if combinations.len() >= total_cells {
                    break;
                }
            }
        }

        if combinations.len() < total_cells {
            return Err(MouselessError::SystemError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Cannot generate enough key combinations for {}x{} grid",
                    self.config.rows, self.config.columns
                ),
            )));
        }

        // Take only the number we need
        combinations.truncate(total_cells);
        Ok(combinations)
    }

    /// Get grid cell by key combination
    pub fn get_cell_by_keys(&self, key_combination: &str) -> Option<&GridCell> {
        self.key_to_cell
            .get(key_combination)
            .and_then(|&index| self.cells.get(index))
    }

    /// Get all grid cells
    pub fn get_cells(&self) -> &[GridCell] {
        &self.cells
    }

    /// Get grid configuration
    pub fn get_config(&self) -> &GridConfig {
        &self.config
    }

    /// Update grid configuration and recalculate
    pub fn update_config(&mut self, config: GridConfig) -> Result<()> {
        self.config = config;
        self.calculate_grid()
    }

    /// Update screen bounds and recalculate grid
    pub fn update_screen_bounds(&mut self, screen_bounds: ScreenBounds) -> Result<()> {
        self.screen_bounds = screen_bounds;
        self.calculate_grid()
    }

    /// Get the center position for a specific grid cell
    pub fn get_cell_center(&self, row: u32, column: u32) -> Option<Position> {
        self.cells
            .iter()
            .find(|cell| cell.row == row && cell.column == column)
            .map(|cell| cell.center_position)
    }

    /// Find the grid cell that contains a given position
    pub fn find_cell_at_position(&self, position: Position) -> Option<&GridCell> {
        self.cells
            .iter()
            .find(|cell| cell.bounds.contains(position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnimationType;

    fn create_test_screen_bounds() -> ScreenBounds {
        ScreenBounds {
            id: 1,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        }
    }

    fn create_test_grid_config() -> GridConfig {
        GridConfig {
            rows: 3,
            columns: 3,
            show_labels: true,
            animation_style: AnimationType::Smooth,
            cell_padding: 2,
            border_width: 1,
            opacity: 0.8,
        }
    }

    #[test]
    fn test_grid_manager_creation() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();

        assert_eq!(manager.get_cells().len(), 9); // 3x3 grid
        assert_eq!(manager.get_config().rows, 3);
        assert_eq!(manager.get_config().columns, 3);
    }

    #[test]
    fn test_grid_cell_calculations() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();
        let cells = manager.get_cells();

        // Check first cell (top-left)
        let first_cell = &cells[0];
        assert_eq!(first_cell.row, 0);
        assert_eq!(first_cell.column, 0);
        assert_eq!(first_cell.bounds.x, 0);
        assert_eq!(first_cell.bounds.y, 0);
        assert_eq!(first_cell.bounds.width, 640); // 1920 / 3
        assert_eq!(first_cell.bounds.height, 360); // 1080 / 3

        // Check center position
        assert_eq!(first_cell.center_position.x, 320); // 640 / 2
        assert_eq!(first_cell.center_position.y, 180); // 360 / 2
    }

    #[test]
    fn test_key_combinations() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();
        let cells = manager.get_cells();

        // Check that all cells have unique key combinations
        let mut key_combinations = std::collections::HashSet::new();
        for cell in cells {
            assert_eq!(cell.key_combination.len(), 2); // Two-key combination
            assert!(key_combinations.insert(cell.key_combination.clone()));
        }

        assert_eq!(key_combinations.len(), 9);
    }

    #[test]
    fn test_get_cell_by_keys() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();
        let first_cell = &manager.get_cells()[0];
        let key_combination = first_cell.key_combination.clone();

        let found_cell = manager.get_cell_by_keys(&key_combination).unwrap();
        assert_eq!(found_cell.row, first_cell.row);
        assert_eq!(found_cell.column, first_cell.column);
    }

    #[test]
    fn test_find_cell_at_position() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();

        // Test position in first cell
        let position = Position::new(100, 100);
        let cell = manager.find_cell_at_position(position).unwrap();
        assert_eq!(cell.row, 0);
        assert_eq!(cell.column, 0);

        // Test position in center cell
        let center_position = Position::new(960, 540); // Center of screen
        let center_cell = manager.find_cell_at_position(center_position).unwrap();
        assert_eq!(center_cell.row, 1);
        assert_eq!(center_cell.column, 1);
    }

    #[test]
    fn test_large_grid() {
        let config = GridConfig {
            rows: 5,
            columns: 6,
            show_labels: true,
            animation_style: AnimationType::Smooth,
            cell_padding: 2,
            border_width: 1,
            opacity: 0.8,
        };
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();

        assert_eq!(manager.get_cells().len(), 30); // 5x6 grid

        // Check that all cells have unique key combinations
        let mut key_combinations = std::collections::HashSet::new();
        for cell in manager.get_cells() {
            assert!(key_combinations.insert(cell.key_combination.clone()));
        }

        assert_eq!(key_combinations.len(), 30);
    }

    #[test]
    fn test_grid_bounds_contain_position() {
        let bounds = GridCellBounds {
            x: 100,
            y: 100,
            width: 200,
            height: 150,
        };

        assert!(bounds.contains(Position::new(150, 150)));
        assert!(bounds.contains(Position::new(100, 100))); // Top-left corner
        assert!(!bounds.contains(Position::new(300, 250))); // Bottom-right corner (exclusive)
        assert!(!bounds.contains(Position::new(50, 50))); // Outside
    }

    #[test]
    fn test_grid_cell_center() {
        let bounds = GridCellBounds {
            x: 100,
            y: 100,
            width: 200,
            height: 150,
        };

        let center = bounds.center();
        assert_eq!(center.x, 200); // 100 + 200/2
        assert_eq!(center.y, 175); // 100 + 150/2
    }

    #[test]
    fn test_key_combination_patterns() {
        let config = create_test_grid_config();
        let screen_bounds = create_test_screen_bounds();

        let manager = GridManager::new(config, screen_bounds).unwrap();
        let cells = manager.get_cells();

        // Test that all key combinations are exactly 2 characters
        for cell in cells {
            assert_eq!(
                cell.key_combination.len(),
                2,
                "Key combination '{}' should be exactly 2 characters",
                cell.key_combination
            );

            // Test that first character is from home row
            let first_char = cell.key_combination.chars().next().unwrap();
            assert!(
                ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'].contains(&first_char),
                "First character '{}' should be from home row",
                first_char
            );

            // Test that second character is from top row
            let second_char = cell.key_combination.chars().nth(1).unwrap();
            assert!(
                ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].contains(&second_char),
                "Second character '{}' should be from top row",
                second_char
            );
        }
    }

    #[test]
    fn test_comprehensive_grid_functionality() {
        let test_cases = vec![
            (2, 2, 4),  // 2x2 = 4 cells
            (3, 3, 9),  // 3x3 = 9 cells
            (4, 4, 16), // 4x4 = 16 cells
            (3, 5, 15), // 3x5 = 15 cells
        ];

        let screen_bounds = create_test_screen_bounds();

        for (rows, columns, expected_cells) in test_cases {
            let config = GridConfig {
                rows,
                columns,
                show_labels: true,
                animation_style: AnimationType::Smooth,
                cell_padding: 2,
                border_width: 1,
                opacity: 0.8,
            };

            let manager = GridManager::new(config, screen_bounds.clone()).unwrap();
            let cells = manager.get_cells();

            // Test correct number of cells
            assert_eq!(
                cells.len(),
                expected_cells,
                "{}x{} grid should have {} cells",
                rows,
                columns,
                expected_cells
            );

            // Test that all key combinations are unique
            let mut key_combinations = std::collections::HashSet::new();
            for cell in cells {
                assert!(
                    key_combinations.insert(cell.key_combination.clone()),
                    "Duplicate key combination: {}",
                    cell.key_combination
                );
            }

            // Test key lookup functionality
            if let Some(first_cell) = manager.get_cells().first() {
                let key_combo = &first_cell.key_combination;
                let found_cell = manager.get_cell_by_keys(key_combo).unwrap();
                assert_eq!(found_cell.key_combination, *key_combo);
                assert_eq!(found_cell.center_position, first_cell.center_position);
            }
        }
    }
}
