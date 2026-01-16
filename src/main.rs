use raylib::prelude::*;

use raylib_framework_testing::game_data::GameData;
use raylib_framework_testing::menu_scene::MenuScene;
use raylib_framework_testing::scenes::SceneManager;

use std::time::Instant;

fn main() {
    let width: i32 = 640;
    let height: i32 = 480;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Raylib Framework Demo")
        .build();


    // create the game data. This prepresents data associated with the human player.
    let mut game_data = GameData::new(width, height);


    // scene manager
    let mut scene_manager = SceneManager::new(&mut rl, Box::new(MenuScene), &mut game_data);

    // store scenes in a Vec. Box<dyn Scene> means a pointer to any type that implements the Scene trait.
    // let mut scenes: Vec<Box<dyn Scene>> = vec![Box::new(MenuScene)];

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
        let mut d = rl.begin_drawing(&thread);
        scene_manager.draw(&mut d, &mut game_data); 

    }
}