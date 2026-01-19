//! Traits for scenes and the scene switch signals.
//! 
use raylib::prelude::*;

use crate::{game_data::GameData, scenes};
///
/// The SceneSwitch enum was conceived with the help of ChatGPT 5.2
/// 
/// These values will signal to the manage that we need to change / update the scene
pub enum SceneSwitch {
    None,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
    Pop,
    PopAndReplace(Box<dyn Scene>), // Pop current scene and replace the one below it
    Quit,
}

///
/// The Scene trait was conceived with the help of ChatGPT 5.2
/// 
/// A manager will call these methods to implement a typical videogame / interactive program loop.
/// 
/// The leading underscore tells the compiler not to complain (warn) if that variable is not read. 
pub trait Scene {
    
    
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    
    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    
    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    
    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData);

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn is_overlay(&self) -> bool {
        false
    }
}


/// SceneManager
/// 
/// This struct controls switching be between different scenes.
pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    quit: bool,

}

impl SceneManager {
    pub fn new(rl: &mut RaylibHandle, initial: Box<dyn Scene>, data: &mut GameData) -> Self {
        let mut mgr = Self {
            scenes: vec![initial],
            quit: false,
        };
        mgr.scenes.last_mut().unwrap().on_enter(rl, data);
        mgr
    }


    pub fn update(&mut self, rl: &mut RaylibHandle, dt: f32, data: &mut GameData) {
        if let Some(scene) = self.scenes.last_mut() {
            let switch = scene.handle_input(rl, data);
            self.apply_switch(switch, rl, data);
        }

        if let Some(scene) = self.scenes.last_mut() {
            let switch = scene.update(dt, data);
            self.apply_switch(switch, rl, data);
        }
    }

    // calls the current scene's [draw] method
    // For overlay scenes (pause/lose), draw the last non-overlay scene (the game) and the overlay
    // For normal scenes, only draw the top scene (which clears background)
    pub fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        // Check if the top scene is an overlay scene
        if let Some(top_scene) = self.scenes.last() {
            if top_scene.is_overlay() {
                // Overlay scene: find and draw the last non-overlay scene (the game), then the overlay
                // This prevents drawing menu scenes that might still be in the stack
                let mut found_game_scene = false;
                for scene in self.scenes.iter().rev() {
                    if !scene.is_overlay() {
                        // Found the last non-overlay scene (the game)
                        scene.draw(d, data);
                        found_game_scene = true;
                        break;
                    }
                }
                // Draw the overlay scene on top
                top_scene.draw(d, data);
            } else {
                // Normal scene: only draw the top scene (it clears background)
                top_scene.draw(d, data);
            }
        }
    }

    // applies a switch returned by either the [handle_input] method or the [update] method.
    pub fn apply_switch(&mut self, switch: SceneSwitch, rl: &mut RaylibHandle, data: &mut GameData) {
        match switch {
            SceneSwitch::None => {},
            SceneSwitch::Push(mut scene) => {
                scene.on_enter(rl, data);
                self.scenes.push(scene);
            },
            SceneSwitch::Replace(mut scene) => {
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
                scene.on_enter(rl, data);
                self.scenes.push(scene);
            }
            SceneSwitch::Pop => {
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
            },
            SceneSwitch::PopAndReplace(mut scene) => {
                
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
                
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
                scene.on_enter(rl, data);
                self.scenes.push(scene);
            },
            SceneSwitch::Quit => {
                self.quit = true;
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit || self.scenes.is_empty() 
    }
}