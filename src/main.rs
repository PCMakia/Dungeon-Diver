//!
//!  Author: Bao Le
//!  Co-author: Khan
//! 
//! 
//! 
//!  
//! 

use raylib_framework_testing::game_data::GameData;
use raylib_framework_testing::menu_scene::TitleScene;
use raylib_framework_testing::scenes::SceneManager;

use std::{fs::OpenOptions, time::Instant};

use std::sync::Arc;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
fn main() {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // tracing_subscriber::Registry::default()
    // .with(stdout_log)
    // .init();

    info!("Application started");

    let file = OpenOptions::new().append(true).create(true).open("debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error {:?}", error),
    };

    let debug_log = tracing_subscriber::fmt::layer().json().with_writer(Arc::new(file));

    tracing_subscriber::Registry::default()
        .with(stdout_log)
        .with(debug_log)
        .init();


    let width: i32 = 1280;
    let height: i32 = 960;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Dungeon Diver V0.5")
        .build();


    // create the game data. This prepresents data associated with the human player.
    let mut game_data = GameData::new(width, height);
    game_data.set_thread(thread);

    game_data.load_sfx();
    
    // Audio device is initialized in GameData::new (do not double-init).

    // scene manager
    let mut scene_manager = SceneManager::new(&mut rl, Box::new(TitleScene), &mut game_data);

    info!("Game Scene started");
    // A variable for the time to calculate update steps in the game. Use for physics and animation.
    let mut last_time = Instant::now();
     
    // the main game / draw loop 
    while !rl.window_should_close() && !scene_manager.should_quit() {
        // update timing.
        let temp = Instant::now();
        let delta = (temp - last_time).as_secs_f32();
        last_time = temp;

        // update and handle user input.
        scene_manager.update(&mut rl, delta, &mut game_data);

        // Draw
        let mut d = rl.begin_drawing(game_data.thread.as_ref().unwrap());
        scene_manager.draw(&mut d, &mut game_data); 

    }
}