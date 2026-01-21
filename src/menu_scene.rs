//! A scene to show a menu
//! 
//! 
use raylib::prelude::*;
// use rand::{self, Rng};
use crate::game_data::{GameData, StageConfig};
use crate::maze_scene::MazeScene;
use crate::scenes::{Scene,SceneSwitch}; 
use crate::utils::*;
use std::path::Path;
use raylib::ffi;
use std::ffi::CString;

/// A start screen or menu screen scene
/// A start screen or menu screen scene
pub struct TitleScene;

impl Scene for TitleScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) {
        // Stop any existing music
        data.stop_music();
        
        // Load and play Lobby music using FFI
        let music_path = resolve_asset_path("assets/SFX/BGM/Lobby/Opening.mp3");
        if Path::new(&music_path).exists() {
            unsafe {
                if let Ok(c_path) = CString::new(music_path.as_str()) {
                    let mut music = ffi::LoadMusicStream(c_path.as_ptr());
                    music.looping = true;
                    ffi::SetMusicVolume(music, 0.0); // Start at 0 for fade-in
                    ffi::PlayMusicStream(music);
                    data.current_music = Some(music);
                    data.music_volume = 0.0;
                    data.music_fade_timer = 0.0;
                }
            }
        }
    }

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
        // Music fade-in is handled in SceneManager
        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
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
    fn on_enter(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) {
        // // Stop any existing music (e.g., stage music from PauseScene)
        // data.stop_music();
        
        // Load and play Lobby music using FFI
        let music_path = resolve_asset_path("assets/SFX/BGM/Lobby/Opening.mp3");
        if Path::new(&music_path).exists() {
            unsafe {
                if let Ok(c_path) = CString::new(music_path.as_str()) {
                    let mut music = ffi::LoadMusicStream(c_path.as_ptr());
                    music.looping = true;
                    ffi::SetMusicVolume(music, 0.0); // Start at 0 for fade-in
                    ffi::PlayMusicStream(music);
                    data.current_music = Some(music);
                    data.music_volume = 0.0;
                    data.music_fade_timer = 0.0;
                }
            }
        }
    }

    fn handle_input(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) -> SceneSwitch {

        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(250.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage I");

                let stage = StageConfig {
                    id: 0,
                    map_path: "assets/maps/mapTest.json".to_string(),
                    music_path: "assets/SFX/BGM/TestStage/synesthesia.mp3".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        // Desert
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(450.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage II");
                let stage = StageConfig {
                    id: 1,
                    map_path: "assets/maps/mapDesert.json".to_string(),
                    music_path: "assets/SFX/BGM/DesertStage/heavens_forbid.ogg".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        // Jungle
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(650.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage III");
                let stage = StageConfig {
                    id: 2,
                    map_path: "assets/maps/mapJungle.json".to_string(),
                    music_path: "assets/SFX/BGM/JungleStage/Kokopelli's Labyrinth Theme ~ Full.ogg".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        // Castle
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(850.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage IV");
                let stage = StageConfig {
                    id: 3,
                    map_path: "assets/maps/mapCastle.json".to_string(),
                    music_path: "assets/SFX/BGM/CastleStage/sinister_abode.wav".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        // Ocean
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(350.0, 300.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage V");

                let stage = StageConfig {
                    id: 4,
                    map_path: "assets/maps/mapOcean.json".to_string(),
                    music_path: "assets/SFX/BGM/OceanStage/song18.mp3".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }   
        }
        // Hell
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(550.0, 300.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage VI");
                let stage = StageConfig {
                    id: 5,
                    map_path: "assets/maps/mapHell.json".to_string(),
                    music_path: "assets/SFX/BGM/HellStage/dark chamber piano.mp3".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        // Flesh
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(750.0, 300.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage VII");
                
                let stage = StageConfig {
                    id: 6,
                    map_path: "assets/maps/mapFlesh.json".to_string(),
                    music_path: "assets/SFX/BGM/FleshStage(Special)/Ruined City Theme.mp3".to_string(),
                };
                data.start_level(stage.id);
                return SceneSwitch::Replace(Box::new(MazeScene::from_map(stage)));
            }
        }
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        // Music fade-in is handled in SceneManager
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        d.draw_text("Dungeon Stages", 450, 95, 50, Color::BLACK);
        // Button Colors
        let button_rect_i = Rectangle::new(250.0, 200.0, 150.0, 50.0);
        let button_color_i = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_i) {
            // Darken green on hover
            Color { r: 0, g: 150, b: 0, a: 255 }
        } else {
            Color::GREEN
        };

        let button_rect_ii = Rectangle::new(450.0, 200.0, 150.0, 50.0);
        let button_color_ii = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_ii) {
            // Darken green on hover
            Color { r: 255, g: 187, b: 10, a: 255 }
        } else {
            Color::SANDYBROWN
        };

        let button_rect_iii = Rectangle::new(650.0, 200.0, 150.0, 50.0);
        let button_color_iii = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_iii) {
            // Darken green on hover
            Color { r: 0, g: 140, b: 0, a: 255 }
        } else {
            Color::DARKGREEN
        };


        let button_rect_iv = Rectangle::new(850.0, 200.0, 150.0, 50.0);
        let button_color_iv = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_iv) {
            // Darken green on hover
            Color { r: 100, g: 89, b: 79, a: 255 }
        } else {
            Color::DARKBROWN
        };

        let button_rect_v = Rectangle::new(350.0, 300.0, 150.0, 50.0);
        let button_color_v = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_v) {
            // Darken green on hover
            Color { r: 98, g: 127, b: 215, a: 255 }
        } else {
            Color::CORNFLOWERBLUE
        };

        let button_rect_vi = Rectangle::new(550.0, 300.0, 150.0, 50.0);
        let button_color_vi = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_vi) {
            // Darken green on hover
            Color { r: 150, g: 0, b: 0, a: 255 }
        } else {
            Color::MAROON
        };

        let button_rect_vii = Rectangle::new(750.0, 300.0, 150.0, 50.0);
        let button_color_vii = if check_collision_point_rect(&d.get_mouse_position(), &button_rect_vii) {
            // Darken green on hover
            Color { r: 223, g: 223, b: 198, a: 255 }
        } else {
            Color::BEIGE
        };
        // stage I
        d.draw_rectangle(250, 200, 150, 50, button_color_i);
        d.draw_text("Stage I", 285, 215, 20, Color::WHEAT);

        // Stage Desert
        d.draw_rectangle(450, 200, 150, 50, button_color_ii);
        d.draw_text("Stage II", 485, 215, 20, Color::BLACK);

        // Stage Jungle
        d.draw_rectangle(650, 200, 150, 50, button_color_iii);
        d.draw_text("Stage III", 685, 215, 20, Color::CYAN);

        // Stage Castle
        d.draw_rectangle(850, 200, 150, 50, button_color_iv);
        d.draw_text("Stage IV", 885, 215, 20, Color::WHITESMOKE);
        
        // Stage Ocean
        d.draw_rectangle(350, 300, 150, 50, button_color_v);
        d.draw_text("Stage V", 385, 315, 20, Color::WHEAT);

        // Stage Hell
        d.draw_rectangle(550, 300, 150, 50, button_color_vi);
        d.draw_text("Stage VI", 585, 315, 20, Color::LIGHTGRAY);

        // Special
        d.draw_rectangle(750, 300, 150, 50, button_color_vii);
        d.draw_text("Stage VII", 785, 315, 20, Color::RED);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) {
        // Stop music when leaving menu (going to game)
        data.stop_music();
    }
}


/// A win screen scene
pub struct WinScene;

impl Scene for WinScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) {
        // Stop current music
        data.stop_music();
        
        // Load and play Winner Full version first using FFI
        let full_path = resolve_asset_path("assets/SFX/BGM/Winner/Continue Theme (Full).wav");
        let loop_path = resolve_asset_path("assets/SFX/BGM/Winner/Continue Theme (Loop).wav");
        
            unsafe {
                if Path::new(&full_path).exists() {
                    if let Ok(c_path) = CString::new(full_path.as_str()) {
                        let mut music = ffi::LoadMusicStream(c_path.as_ptr());
                        music.looping = false; // Don't loop the full version
                        ffi::SetMusicVolume(music, 1.0); // Start at full volume (no fade-in for win)
                        ffi::PlayMusicStream(music);
                        data.current_music = Some(music);
                        data.music_volume = 1.0;
                        data.music_fade_timer = data.music_fade_duration; // Skip fade-in
                        data.win_music_full_played = false;
                    }
                }
                
                // Pre-load the loop version
                if Path::new(&loop_path).exists() {
                    if let Ok(c_path) = CString::new(loop_path.as_str()) {
                        let mut loop_music = ffi::LoadMusicStream(c_path.as_ptr());
                        loop_music.looping = true;
                        ffi::SetMusicVolume(loop_music, 1.0);
                        data.win_music_loop = Some(loop_music);
                    }
                }
            }
    }

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {

        
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            // Button rectangle for "Back to Menu" 
            let button_rect = Rectangle::new(450.0, 620.0, 300.0, 60.0);
            if check_collision_point_rect(&click, &button_rect) {
                println!("Back to menu clicked");
                // Replace WinScene to return to MenuScene
                return SceneSwitch::Replace(Box::new(MenuScene));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        // Music transition handled in GameData::update_music_fade
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        // Title
        d.draw_text("Level Complete!", 400, 300, 60, Color::BLACK);
        let stage = data.current_stage;
        let best = data.stage_high_scores[stage];

        d.draw_text(
            &format!(
                "Stage {}   Best: {}",
                stage + 1,
                best
            ),
            450,
            400,
            35,
            Color::DARKVIOLET,
        );
                
        // Score display
        let score_message = format!("Final Score: {}", data.points);
        d.draw_text(score_message.as_str(), 500, 470, 30, Color::BLACK);
        
        // Time display
        if let Some(elapsed) = data.get_elapsed_time() {
            let minutes = (elapsed as u32) / 60;
            let seconds = (elapsed as u32) % 60;
            let milliseconds = ((elapsed % 1.0) * 100.0) as u32;
            let time_message = format!("Time: {:02}:{:02}.{:02}", minutes, seconds, milliseconds);
            d.draw_text(time_message.as_str(), 500, 520, 30, Color::BLACK);
        } else {
            d.draw_text("Time: --:--", 500, 520, 30, Color::GRAY);
        }
        
        // Back to Menu button
        let button_rect = Rectangle::new(450.0, 620.0, 300.0, 60.0);
        let button_color = if check_collision_point_rect(&d.get_mouse_position(), &button_rect) {
            // Darken green on hover
            Color { r: 0, g: 150, b: 0, a: 255 }
        } else {
            Color::GREEN
        };
        d.draw_rectangle(450, 620, 300, 60, button_color);
        d.draw_text("Back to Menu", 515, 640, 25, Color::WHITE);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) {
        // Stop win music when leaving
        data.stop_music();
    }
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
        // controller support
        if _rl.is_gamepad_available(0) {
            if _rl.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT) {
                return SceneSwitch::Pop;
            }
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
    stage: StageConfig,
}

impl LoseScene {
    pub fn new(stage: StageConfig) -> Self {
        Self { stage }
    }
}

impl Scene for LoseScene {
    fn is_overlay(&self) -> bool {
        true
    }
    
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) -> SceneSwitch {
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            
            // Calculate button positions to match draw() coordinates
            let box_x = (data.screen_width - 500) / 2;
            let box_y = (data.screen_height - 400) / 2;
            
            let retry_rect = Rectangle::new((box_x + 50) as f32, (box_y + 200) as f32, 400.0, 60.0);
            if check_collision_point_rect(&click, &retry_rect) {
                // Reset score and timer for retry
                data.points = 0;
                data.level_start_time = None;
                data.level_completion_time = None;
                
                use crate::maze_scene::MazeScene;
                data.start_level(data.current_stage);
                return SceneSwitch::PopAndReplace(Box::new(MazeScene::from_map(self.stage.clone())));
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