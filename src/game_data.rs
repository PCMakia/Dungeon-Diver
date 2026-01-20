//! The data for each game session. 
//! 
//! You could also store data associated with each human player here.
//! We could also store the player's gamepad_id here.

use raylib::prelude::*;
use raylib::ffi;
use std::time::Instant;

use serde::{Serialize, Deserialize};

use std::ffi::CString;
use std::path::Path;

// Music implementation
impl GameData {
    pub fn play_stage_music(&mut self, path: &str) {
        self.stop_music();

        if !Path::new(path).exists() {
            eprintln!("Music not found: {}", path);
            return;
        }

        unsafe {
            if let Ok(c_path) = CString::new(path) {
                let mut music = ffi::LoadMusicStream(c_path.as_ptr());
                music.looping = true;
                ffi::SetMusicVolume(music, 0.0);
                ffi::PlayMusicStream(music);

                self.current_music = Some(music);
                self.music_volume = 0.0;
                self.music_fade_timer = 0.0;
            }
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub stage_high_scores: [u32; 7],
}


impl Default for SaveData {
    fn default() -> Self {
        Self {
            stage_high_scores: [0; 7],
        }
    }
}
#[derive(Clone)]
pub struct StageConfig {
    pub id: usize,
    pub map_path: String,
    pub music_path: String,
}


pub struct GameData {
    pub points: u32,

    // Local scores
    pub current_stage: usize,          
    pub stage_high_scores: [u32; 7],

    pub screen_width: i32,
    pub screen_height: i32,
    pub thread: Option<RaylibThread>,
    // Timing for level completion
    pub level_start_time: Option<Instant>,
    pub level_completion_time: Option<Instant>,
    // Music state - using FFI Music directly
    pub current_music: Option<ffi::Music>,
    pub music_volume: f32,
    pub music_fade_timer: f32,
    pub music_fade_duration: f32,
    // For WinScene: track if we're playing full or loop version
    pub win_music_full_played: bool,
    pub win_music_loop: Option<ffi::Music>,

    
}

impl GameData {
    pub fn new(width: i32, height: i32) -> Self {
        let save = load_save();

        Self {
            points: 0,
            current_stage: 0,
            stage_high_scores: save.stage_high_scores,

            screen_width: width,
            screen_height: height,
            thread: None,

            level_start_time: None,
            level_completion_time: None,

            current_music: None,
            music_volume: 0.0,
            music_fade_timer: 0.0,
            music_fade_duration: 1.0,

            win_music_full_played: false,
            win_music_loop: None,
        }
    }
    
    pub fn set_thread(&mut self, thread: RaylibThread) {
        self.thread = Some(thread);
    }

    /// Add an arbitrary number of points to the player's score.
    pub fn add_points(&mut self, amount: u32) {
        self.points = self.points.saturating_add(amount);
    }
    
    /// Start timing a level
    pub fn start_level_timer(&mut self) {
        self.level_start_time = Some(Instant::now());
        self.level_completion_time = None;
    }
    pub fn start_level(&mut self, stage: usize) {
        self.current_stage = stage;
        self.points = 0;
        self.start_level_timer();
    }

    
    /// Record level completion time
    pub fn complete_level(&mut self) {
        self.level_completion_time = Some(Instant::now());
        let stage = self.current_stage;

        if self.points > self.stage_high_scores[stage] {
            self.stage_high_scores[stage] = self.points;

            save_save(&SaveData {
                stage_high_scores: self.stage_high_scores,
            });
        }
    }

    
    
    /// Get elapsed time in seconds (returns None if level hasn't started or completed)
    pub fn get_elapsed_time(&self) -> Option<f32> {
        if let (Some(start), Some(completion)) = (self.level_start_time, self.level_completion_time) {
            Some((completion - start).as_secs_f32())
        } else {
            None
        }
    }
    
    /// Stop current music and clean up
    pub fn stop_music(&mut self) {
        unsafe {
            if let Some(music) = self.current_music.take() {
                ffi::StopMusicStream(music);
                ffi::UnloadMusicStream(music);
            }
            if let Some(loop_music) = self.win_music_loop.take() {
                ffi::StopMusicStream(loop_music);
                ffi::UnloadMusicStream(loop_music);
            }
        }
        self.music_volume = 0.0;
        self.music_fade_timer = 0.0;
        self.win_music_full_played = false;
    }
    
    /// Update music volume fade-in
    pub fn update_music_fade(&mut self, dt: f32) {
        unsafe {
            // Update fade-in timer
            if self.music_fade_timer < self.music_fade_duration {
                self.music_fade_timer += dt;
                self.music_volume = (self.music_fade_timer / self.music_fade_duration).min(1.0);
            } else {
                self.music_volume = 1.0;
            }
            
            // Handle win scene transition from full to loop
            if let Some(mut music) = self.current_music.take() {
                if !self.win_music_full_played {
                    // Check if full version finished
                    if !ffi::IsMusicStreamPlaying(music) {
                        // Full version finished, switch to loop
                        if let Some(mut loop_music) = self.win_music_loop.take() {
                            ffi::StopMusicStream(music);
                            ffi::UnloadMusicStream(music);
                            loop_music.looping = true;
                            ffi::SetMusicVolume(loop_music, self.music_volume);
                            ffi::PlayMusicStream(loop_music);
                            self.current_music = Some(loop_music);
                            self.win_music_full_played = true;
                        } else {
                            // No loop music, put music back
                            self.current_music = Some(music);
                        }
                    } else {
                        // Update volume and stream for full version
                        ffi::SetMusicVolume(music, self.music_volume);
                        ffi::UpdateMusicStream(music);
                        self.current_music = Some(music);
                    }
                } else {
                    // Normal music or win loop - update volume and stream
                    ffi::SetMusicVolume(music, self.music_volume);
                    ffi::UpdateMusicStream(music);
                    
                    // Loop normal music (except WinScene)
                    if !music.looping && !ffi::IsMusicStreamPlaying(music) && self.win_music_loop.is_none() {
                        ffi::PlayMusicStream(music);
                    }
                    self.current_music = Some(music);
                }
            }
            
            // Update win loop music if it's still separate (shouldn't happen, but safety check)
            if let Some(mut loop_music) = self.win_music_loop.take() {
                ffi::SetMusicVolume(loop_music, self.music_volume);
                ffi::UpdateMusicStream(loop_music);
                
                // Loop the win loop music
                if !ffi::IsMusicStreamPlaying(loop_music) {
                    ffi::PlayMusicStream(loop_music);
                }
                self.win_music_loop = Some(loop_music);
            }
        }
    }
}


use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::PathBuf;

fn save_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();
    path.push("dungeon_diver");
    create_dir_all(&path).ok();
    path.push("save_score.json");
    path
}

pub fn load_save() -> SaveData {
    let path = save_path();

    if let Ok(mut file) = File::open(path) {
        let mut s = String::new();
        file.read_to_string(&mut s).ok();
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        SaveData::default()
    }
}

pub fn save_save(data: &SaveData) {
    let path = save_path();
    if let Ok(mut file) = File::create(path) {
        let _ = file.write_all(
            serde_json::to_string_pretty(data).unwrap().as_bytes()
        );
    }
}