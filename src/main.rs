pub mod core;
pub mod frontend;

use crate::core::entities::Piece;

use crate::core::coord::HexCoord;

use crate::core::board::Board;
use crate::core::entities::Player;
use crate::core::game::Game;

use frontend::frontend::Frontend;
use macroquad::prelude::*;
use macroquad::window::Conf;

fn window_conf() -> Conf {
    Conf {
        window_title: "yinsh".to_owned(),
        window_width: 1024,
        window_height: 1024,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut board = Board::new();
    let font = load_ttf_font("./assets/MerriweatherSans-VariableFont_wght.ttf")
        .await
        .unwrap();

    let frontend = Frontend::new(&board, font, 1024, 1024, 2., 2.);
    let mut game = Game::new(Player::White, Box::new(frontend), board, 5);

    loop {
        game.tick();
        next_frame().await
    }
}
