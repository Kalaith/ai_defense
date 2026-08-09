//! Last Assembly entry point: window setup and the macroquad main loop.

mod data;
mod engine;
mod game;
mod save;
mod state;
mod ui;

use game::Game;
use macroquad::prelude::*;
use macroquad_toolkit::capture;
use macroquad_toolkit::colors::dark;

fn window_conf() -> Conf {
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("AI_DEFENSE");

    // Built by hand (not capture::capture_window_conf) to keep the
    // sample_count: 0 / high_dpi: false fields this game already relies on;
    // window_width/height still honor the AI_DEFENSE_WINDOW_* overrides so the
    // capture harness can pick a deterministic framebuffer size.
    Conf {
        window_title: "Last Assembly".to_owned(),
        window_width: capture::env_i32("AI_DEFENSE_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("AI_DEFENSE_WINDOW_HEIGHT", 720),
        window_resizable: true,
        sample_count: 0,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when AI_DEFENSE_CAPTURE_PATH is set, seed a scene,
    // render deterministic frames, write a PNG, and exit. See
    // docs/screenshot_capture_harness_guide.md.
    if let Some(configs) = capture::CaptureConfig::all_from_env("AI_DEFENSE") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                clear_background(dark::BACKGROUND);
                game.update();
                game.draw();
            })
            .await;
        }
        return;
    }

    loop {
        clear_background(dark::BACKGROUND);
        game.update();
        game.draw();
        next_frame().await;
    }
}
