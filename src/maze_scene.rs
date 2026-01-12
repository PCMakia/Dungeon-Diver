//! The maze game scene
//! 
//! This represents a maze game where the player navigates from start to exit.

use raylib::prelude::*;

use crate::menu_scene::WinScene;
use crate::scenes::{Scene, SceneSwitch};
use crate::game_data::GameData;
use crate::utils::*;
use rand::Rng;

// Define cell types for our maze
#[derive(Clone, Copy, PartialEq)]
pub enum CellType {
    Wall,
    Path,
    Start,
    Exit,
}

pub struct MazeScene {
    // Maze grid dimensions
    grid_width: usize,
    grid_height: usize,
    cell_size: i32,
    
    // Maze data
    grid: Vec<Vec<CellType>>,
    
    // Player position in grid coordinates
    player_x: usize,
    player_y: usize,
    
    // Player movement
    player_speed: f32,
}

impl MazeScene {
    pub fn new(width: i32, height: i32) -> Self {
        let cell_size = 30; // Size of each cell in pixels
        let grid_width = (width / cell_size) as usize;
        let grid_height = (height / cell_size) as usize;
        
        // Create a simple maze for now
        let mut grid = vec![vec![CellType::Wall; grid_width]; grid_height];
        
        // Generate a simple maze
        Self::generate_simple_maze(&mut grid);
        
        // Find start position (first path cell)
        let mut player_x = 1;
        let mut player_y = 1;
        for y in 0..grid_height {
            for x in 0..grid_width {
                if grid[y][x] == CellType::Start {
                    player_x = x;
                    player_y = y;
                    break;
                }
            }
        }
        
        Self {
            grid_width,
            grid_height,
            cell_size,
            grid,
            player_x,
            player_y,
            player_speed: 5.0, // Grid cells per second
        }
    }
    
    // Generate a simple maze with walls around the edges and some random walls
    fn generate_simple_maze(grid: &mut Vec<Vec<CellType>>) {
        let height = grid.len();
        let width = grid[0].len();
        let mut rng = rand::thread_rng();
        
        // Start with all paths
        for y in 0..height {
            for x in 0..width {
                grid[y][x] = CellType::Path;
            }
        }
        
        // Add walls around the edges
        for y in 0..height {
            grid[y][0] = CellType::Wall;
            grid[y][width-1] = CellType::Wall;
        }
        
        for x in 0..width {
            grid[0][x] = CellType::Wall;
            grid[height-1][x] = CellType::Wall;
        }
        
        // Add some random walls (simple maze generation)
        for _ in 0..((width * height) / 5) {
            let x = rng.gen_range(1..width-1);
            let y = rng.gen_range(1..height-1);
            grid[y][x] = CellType::Wall;
        }
        
        // Ensure there's a path through the maze (this is a very simple approach)
        for y in 1..height-1 {
            if y % 2 == 0 {
                grid[y][width/2] = CellType::Path;
            }
        }
        
        // Set start and exit points
        grid[1][1] = CellType::Start;
        grid[height-2][width-2] = CellType::Exit;
    }
    
    // Check if a move to the given position is valid
    fn is_valid_move(&self, x: usize, y: usize) -> bool {
        if x >= self.grid_width || y >= self.grid_height {
            return false;
        }
        
        self.grid[y][x] != CellType::Wall
    }
}

impl Scene for MazeScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {
        // Reset score when entering the maze
        _data.points = 0;
    }

    fn handle_input(&mut self, rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        // Handle player movement with arrow keys or WASD
        let mut new_x = self.player_x;
        let mut new_y = self.player_y;
        
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
            new_x += 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
            if new_x > 0 {
                new_x -= 1;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
            new_y += 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
            if new_y > 0 {
                new_y -= 1;
            }
        }
        
        // Check if the move is valid and update position
        if self.is_valid_move(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, data: &mut GameData) -> SceneSwitch {
        // Check if player has reached the exit
        if self.grid[self.player_y][self.player_x] == CellType::Exit {
            // Add points for completing the maze
            data.score();
            return SceneSwitch::Push(Box::new(WinScene));
        }
        
        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        // Draw the maze
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let cell_x = (x as i32) * self.cell_size;
                let cell_y = (y as i32) * self.cell_size;
                
                match self.grid[y][x] {
                    CellType::Wall => {
                        d.draw_rectangle(cell_x, cell_y, self.cell_size, self.cell_size, Color::BLACK);
                    },
                    CellType::Path => {
                        d.draw_rectangle(cell_x, cell_y, self.cell_size, self.cell_size, Color::WHITE);
                    },
                    CellType::Start => {
                        d.draw_rectangle(cell_x, cell_y, self.cell_size, self.cell_size, Color::GREEN);
                    },
                    CellType::Exit => {
                        d.draw_rectangle(cell_x, cell_y, self.cell_size, self.cell_size, Color::RED);
                    }
                }
                
                // Draw grid lines
                d.draw_rectangle_lines(cell_x, cell_y, self.cell_size, self.cell_size, Color::GRAY);
            }
        }
        
        // Draw player
        let player_screen_x = (self.player_x as i32) * self.cell_size + (self.cell_size / 2);
        let player_screen_y = (self.player_y as i32) * self.cell_size + (self.cell_size / 2);
        d.draw_circle(player_screen_x, player_screen_y, (self.cell_size as f32) * 0.4, Color::BLUE);
        
        // Draw score
        let message = format!("Score: {}", data.points);
        d.draw_text(message.as_str(), 10, data.screen_height - 25, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}
