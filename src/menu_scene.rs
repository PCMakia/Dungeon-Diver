//! A scene to show a menu
//! 
//! 
use raylib::prelude::*;
// use rand::{self, Rng};

use crate::game_data::GameData;
use crate::maze_scene::MazeScene;
use crate::scenes::{Scene,SceneSwitch}; 
use crate::utils::*;

/// A start screen or menu screen scene
/// A start screen or menu screen scene
pub struct TitleScene;

impl Scene for TitleScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            // Button rectangle: centered in bottom half (480-960)
            let button_rect = Rectangle::new(490.0, 645.0, 300.0, 150.0);
            if check_collision_point_rect(&click, &button_rect) {
                return SceneSwitch::Replace(Box::new(MenuScene));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        // Draw title: centered in top half (0-480)
        d.draw_text("Dungeon Diver", 385, 215, 70, Color::BLACK);
        
        // Draw "Start" button: centered in bottom half (480-960)
        d.draw_rectangle(490, 645, 300, 150, Color::GREEN);
        d.draw_text("Start", 600, 700, 30, Color::WHITE);  // Centered inside button
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}

pub struct MenuScene;

impl Scene for MenuScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) -> SceneSwitch {

        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(200.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage");
                return SceneSwitch::Push(Box::new(MazeScene::from_map("assets/maps/mapTest.json".to_string())));


            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        d.draw_text("Dungeon Stages", 450, 95, 50, Color::BLACK);
        d.draw_rectangle(200, 200, 150, 50, Color::GREEN);
        d.draw_text("Stage I", 235, 215, 20, Color::WHEAT);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}


/// A win screen scene
pub struct WinScene;

impl Scene for WinScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {

        
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            // Button rectangle for "Back to Menu" 
            let button_rect = Rectangle::new(450.0, 550.0, 300.0, 60.0);
            if check_collision_point_rect(&click, &button_rect) {
                println!("Back to menu clicked");
                // Pop WinScene to return to MenuScene
                return SceneSwitch::Pop;
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        // Title
        d.draw_text("Level Complete!", 400, 300, 50, Color::BLACK);
        
        // Score display
        let score_message = format!("Final Score: {}", data.points);
        d.draw_text(score_message.as_str(), 500, 400, 30, Color::BLACK);
        
        // Time display
        if let Some(elapsed) = data.get_elapsed_time() {
            let minutes = (elapsed as u32) / 60;
            let seconds = (elapsed as u32) % 60;
            let milliseconds = ((elapsed % 1.0) * 100.0) as u32;
            let time_message = format!("Time: {:02}:{:02}.{:02}", minutes, seconds, milliseconds);
            d.draw_text(time_message.as_str(), 500, 450, 30, Color::BLACK);
        } else {
            d.draw_text("Time: --:--", 500, 450, 30, Color::GRAY);
        }
        
        // Back to Menu button
        let button_rect = Rectangle::new(450.0, 550.0, 300.0, 60.0);
        let button_color = if check_collision_point_rect(&d.get_mouse_position(), &button_rect) {
            // Darken green on hover
            Color { r: 0, g: 150, b: 0, a: 255 }
        } else {
            Color::GREEN
        };
        d.draw_rectangle(450, 550, 300, 60, button_color);
        d.draw_text("Back to Menu", 515, 570, 25, Color::WHITE);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}      


pub struct PauseScene;

impl Scene for PauseScene {
    fn is_overlay(&self) -> bool {
        true
    }
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        // ESC or P to resume (pop pause scene)
        if _rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) || _rl.is_key_pressed(KeyboardKey::KEY_P) {
            return SceneSwitch::Pop;
        }
        
        // Button clicks
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            
            // Calculate button positions to match draw() coordinates
            let box_x = (_data.screen_width - 500) / 2;
            let box_y = (_data.screen_height - 400) / 2;
            
            // Resume button - pop pause scene to return to game
            let resume_rect = Rectangle::new((box_x + 50) as f32, (box_y + 200) as f32, 400.0, 60.0);
            if check_collision_point_rect(&click, &resume_rect) {
                return SceneSwitch::Pop;
            }
            
            // Back to Menu button - replace pause scene with menu scene
            // This will pop pause scene and push menu scene, leaving game scene underneath
            let menu_rect = Rectangle::new((box_x + 50) as f32, (box_y + 280) as f32, 400.0, 60.0);
            if check_collision_point_rect(&click, &menu_rect) {
                use crate::menu_scene::MenuScene;
                return SceneSwitch::Replace(Box::new(MenuScene));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        // Draw semi-transparent overlay
        let overlay_color = Color { r: 0, g: 0, b: 0, a: 128 }; // Black with 50% opacity
        d.draw_rectangle(0, 0, data.screen_width, data.screen_height, overlay_color);
        
        // Draw solid notification box
        let box_x = (data.screen_width - 500) / 2;
        let box_y = (data.screen_height - 400) / 2;
        d.draw_rectangle(box_x, box_y, 500, 400, Color::DARKGRAY);
        d.draw_rectangle_lines(box_x, box_y, 500, 400, Color::WHITE);
        
        // Title
        d.draw_text("PAUSED", box_x + 150, box_y + 30, 50, Color::WHITE);
        
        // Score display
        let score_message = format!("Current Score: {}", data.points);
        d.draw_text(score_message.as_str(), box_x + 50, box_y + 120, 30, Color::WHITE);
        
        // Resume button
        let resume_rect = Rectangle::new((box_x + 50) as f32, (box_y + 200) as f32, 400.0, 60.0);
        let resume_color = if check_collision_point_rect(&d.get_mouse_position(), &resume_rect) {
            Color::GREEN
        } else {
            Color { r: 0, g: 200, b: 0, a: 255 }
        };
        d.draw_rectangle(box_x + 50, box_y + 200, 400, 60, resume_color);
        d.draw_text("Resume", box_x + 150, box_y + 215, 30, Color::WHITE);
        
        // Back to Menu button
        let menu_rect = Rectangle::new((box_x + 50) as f32, (box_y + 280) as f32, 400.0, 60.0);
        let menu_color = if check_collision_point_rect(&d.get_mouse_position(), &menu_rect) {
            Color { r: 200, g: 130, b: 0, a: 255 }
        } else {
            Color { r: 255, g: 165, b: 0, a: 255 }
        };
        d.draw_rectangle(box_x + 50, box_y + 280, 400, 60, menu_color);
        d.draw_text("Back to Menu", box_x + 150, box_y + 295, 30, Color::WHITE);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}      

pub struct LoseScene {
    map_path: String, // Store the map path to restart the level
}

impl LoseScene {
    pub fn new(map_path: String) -> Self {
        Self { map_path }
    }
}

impl Scene for LoseScene {
    fn is_overlay(&self) -> bool {
        true
    }
    
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            
            // Calculate button positions to match draw() coordinates
            let box_x = (_data.screen_width - 500) / 2;
            let box_y = (_data.screen_height - 400) / 2;
            
            let retry_rect = Rectangle::new((box_x + 50) as f32, (box_y + 200) as f32, 400.0, 60.0);
            if check_collision_point_rect(&click, &retry_rect) {
                use crate::maze_scene::MazeScene;
                return SceneSwitch::PopAndReplace(Box::new(MazeScene::from_map(self.map_path.clone())));
            }
            
            // Back to Menu button - replace lose scene with menu scene
            // This will pop lose scene and push menu scene, leaving game scene underneath
            // The menu scene will be on top
            let menu_rect = Rectangle::new((box_x + 50) as f32, (box_y + 280) as f32, 400.0, 60.0);
            if check_collision_point_rect(&click, &menu_rect) {
                use crate::menu_scene::MenuScene;
                return SceneSwitch::Replace(Box::new(MenuScene));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        // Draw semi-transparent overlay
        let overlay_color = Color { r: 0, g: 0, b: 0, a: 128 }; 
        d.draw_rectangle(0, 0, data.screen_width, data.screen_height, overlay_color);
        
        // Draw solid notification box
        let box_x = (data.screen_width - 500) / 2;
        let box_y = (data.screen_height - 400) / 2;
        d.draw_rectangle(box_x, box_y, 500, 400, Color::DARKGRAY);
        d.draw_rectangle_lines(box_x, box_y, 500, 400, Color::RED);
        
        // Title
        d.draw_text("YOU DIED", box_x + 120, box_y + 30, 50, Color::RED);
        
        // Score display
        let score_message = format!("Final Score: {}", data.points);
        d.draw_text(score_message.as_str(), box_x + 50, box_y + 120, 30, Color::WHITE);
        
        // Time display
        if let Some(elapsed) = data.get_elapsed_time() {
            let minutes = (elapsed as u32) / 60;
            let seconds = (elapsed as u32) % 60;
            let milliseconds = ((elapsed % 1.0) * 100.0) as u32;
            let time_message = format!("Time: {:02}:{:02}.{:02}", minutes, seconds, milliseconds);
            d.draw_text(time_message.as_str(), box_x + 50, box_y + 160, 25, Color::WHITE);
        }
        
        // Retry button
        let retry_rect = Rectangle::new((box_x + 50) as f32, (box_y + 200) as f32, 400.0, 60.0);
        let retry_color = if check_collision_point_rect(&d.get_mouse_position(), &retry_rect) {
            Color::GREEN
        } else {
            Color { r: 0, g: 200, b: 0, a: 255 }
        };
        d.draw_rectangle(box_x + 50, box_y + 200, 400, 60, retry_color);
        d.draw_text("Retry Level", box_x + 150, box_y + 215, 30, Color::WHITE);
        
        // Back to Menu button
        let menu_rect = Rectangle::new((box_x + 50) as f32, (box_y + 280) as f32, 400.0, 60.0);
        let menu_color = if check_collision_point_rect(&d.get_mouse_position(), &menu_rect) {
            Color { r: 200, g: 130, b: 0, a: 255 }
        } else {
            Color { r: 255, g: 165, b: 0, a: 255 }
        };
        d.draw_rectangle(box_x + 50, box_y + 280, 400, 60, menu_color);
        d.draw_text("Back to Menu", box_x + 150, box_y + 295, 30, Color::WHITE);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}     