mod data;
mod engine;
mod game;
mod save;
mod state;
mod ui;

use game::Game;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;

fn window_conf() -> Conf {
    Conf {
        window_title: "Last Assembly".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        sample_count: 0,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        clear_background(dark::BACKGROUND);
        game.update();
        game.draw();
        next_frame().await;
    }
}
