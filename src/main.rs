//!
//!  Author: Bao Le
//!  Co-author (map): Khan
//! 
//! 
//! 
//!  
//! 

use raylib_framework_testing::game_data::GameData;
use raylib_framework_testing::menu_scene::TitleScene;
use raylib_framework_testing::scenes::SceneManager;

#[cfg(target_os = "emscripten")]
use std::cell::RefCell;
use std::time::Instant;

#[cfg(not(target_os = "emscripten"))]
use std::fs::OpenOptions;
#[cfg(not(target_os = "emscripten"))]
use std::sync::Arc;

use tracing::info;
use tracing_subscriber::prelude::*;

#[cfg(target_os = "emscripten")]
struct EmscriptenGameState {
    rl: raylib::prelude::RaylibHandle,
    game_data: GameData,
    scene_manager: SceneManager,
    last_time: Instant,
}

#[cfg(target_os = "emscripten")]
thread_local! {
    static EMSCRIPTEN_GAME: RefCell<Option<EmscriptenGameState>> = RefCell::new(None);
}

#[cfg(target_os = "emscripten")]
unsafe extern "C" {
    fn emscripten_set_main_loop(func: extern "C" fn(), fps: i32, simulate_infinite_loop: i32);
    fn emscripten_cancel_main_loop();
}

#[cfg(target_os = "emscripten")]
extern "C" fn emscripten_main_loop_frame() {
    EMSCRIPTEN_GAME.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(game) = slot.as_mut() else {
            return;
        };
        // On PLATFORM_WEB, raylib's WindowShouldClose() can call emscripten_sleep(), which
        // requires ASYNCIFY. We use emscripten_set_main_loop instead, so avoid that path.
        if game.scene_manager.should_quit() {
            unsafe { emscripten_cancel_main_loop() };
            *slot = None;
            return;
        }
        let temp = Instant::now();
        // Clamp large frame gaps (tab switches / GC pauses) to avoid giant simulation jumps.
        let delta = (temp - game.last_time).as_secs_f32().min(0.05);
        game.last_time = temp;
        game.scene_manager.update(&mut game.rl, delta, &mut game.game_data);
        let mut d = game
            .rl
            .begin_drawing(game.game_data.thread.as_ref().unwrap());
        game.scene_manager.draw(&mut d, &mut game.game_data);
    });
}

fn init_tracing() {
    #[cfg(target_os = "emscripten")]
    {
        let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).try_init();
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        let stdout_log = tracing_subscriber::fmt::layer().pretty();
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("debug.log")
            .expect("debug.log");
        let debug_log = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(Arc::new(file));
        tracing_subscriber::Registry::default()
            .with(stdout_log)
            .with(debug_log)
            .init();
    }
}

fn main() {
    init_tracing();

    info!("Application started");

    let width: i32 = 1280;
    let height: i32 = 960;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Dungeon Diver V0.5")
        .build();

    // Web: EndDrawing() uses WaitTime() → nanosleep(), which blocks the JS main thread.
    #[cfg(target_os = "emscripten")]
    rl.set_target_fps(0);

    let mut game_data = GameData::new(width, height);
    game_data.set_thread(thread);
    game_data.load_sfx();

    let mut scene_manager = SceneManager::new(&mut rl, Box::new(TitleScene), &mut game_data);

    info!("Game Scene started");
    let last_time = Instant::now();

    #[cfg(target_os = "emscripten")]
    {
        // Drive frames with requestAnimationFrame so the tab stays responsive (refresh, DevTools).
        EMSCRIPTEN_GAME.with(|slot| {
            *slot.borrow_mut() = Some(EmscriptenGameState {
                rl,
                game_data,
                scene_manager,
                last_time,
            });
        });
        // Keep runtime alive under Emscripten. If simulate_infinite_loop=0, main can return and
        // shut down before rAF frames execute, which leaves a black canvas.
        unsafe { emscripten_set_main_loop(emscripten_main_loop_frame, 0, 1) };
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        let mut last_time = last_time;
        while !rl.window_should_close() && !scene_manager.should_quit() {
            let temp = Instant::now();
            // Clamp large frame gaps (tab switches / GC pauses) to avoid giant simulation jumps.
            let delta = (temp - last_time).as_secs_f32().min(0.05);
            last_time = temp;
            scene_manager.update(&mut rl, delta, &mut game_data);
            let mut d = rl.begin_drawing(game_data.thread.as_ref().unwrap());
            scene_manager.draw(&mut d, &mut game_data);
        }
    }
}