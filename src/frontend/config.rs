use macroquad::prelude::*;

use crate::core::entities::Player;
// colors
pub const GRID_LINE_COLOR: Color = Color { r: 1., g: 1., b: 1., a: 0.7};
pub const BACKGROUND_COLOR: Color = WHITE;

pub const BLACK_PLAYER_COLOR: Color = Color {r: 0./255., g: 92./255., b: 155./255., a: 1.};
pub const WHITE_PLAYER_COLOR: Color = WHITE;
pub const DARK_BORDER_COLOR: Color = DARKGRAY;
pub const LIGHT_BORDER_COLOR: Color = Color {r: 0./255., g: 40./255., b: 60./255., a: 1.};
pub const HOVER_COLOR: Color = ORANGE;
pub const SELECT_COLOR: Color = ORANGE;
pub const REMOVE_COLOR: Color = ORANGE;
pub const LEGAL_MOVE_MARKER_COLOR: Color = Color { r: 0., g: 0.2, b: 0.8, a: 0.7};

pub const RUN_INDICATOR_COLOR: Color = Color { r: 0.98, g: 0.662, b: 0.186, a: 0.1};
pub const RUN_INDICATOR_LINE_COLOR: Color = DARKGRAY;

pub const RUN_INDICATOR_COLOR_HOVER: Color = WHITE;
pub const RUN_INDICATOR_LINE_COLOR_HOVER: Color = DARKGRAY;

pub const RESTART_WINDOW_BG_COLOR: Color = Color { r: 213./255., g: 240./255., b: 245./255., a: 1.0 };
pub const RESTART_WINDOW_STATUS_COLOR: Color = DARKGRAY;
pub const RESTART_WINDOW_SCORE_COLOR: Color = BLACK_PLAYER_COLOR;
pub const BUTTON_DEFAULT_COLOR: Color = WHITE;
pub const BUTTON_HOVER_COLOR: Color = ORANGE;
pub const BUTTON_BORDER_COLOR: Color = BLACK;
pub const BUTTON_TEXT_COLOR: Color = DARKBLUE;

// geometry
pub const GRID_LNE_WIDTH: f32 = 0.02;
pub const MARKER_BORDER_WIDTH: f32 = 0.02;
pub const RING_BORDER_WIDTH: f32 = 0.03;
pub const RING_INNER_RADIUS: f32 = 0.25;
pub const RING_OUTER_RADIUS: f32 = 0.4;
pub const MARKER_RADIUS: f32 = 0.18;
pub const LEGAL_MOVE_MARKER_RADIUS: f32 = 0.075;
pub const RING_SEGMENTS: u16 = 32;

pub const RUN_INDICATOR_CIRCLE_SEGMENTS: u16 = 16;

pub const RESTART_WINDOW_WIDTH: f32 = 6.0;
pub const RESTART_WINDOW_HEIGHT: f32 = 3.0;

pub const RESTART_WINDOW_STATUS_FONTSIZE: f32 = 0.5;
pub const RESTART_WINDOW_SCORE_FONTSIZE: f32 = 0.5;

pub const BUTTON_FONT_SIZE: f32 = 0.25;
pub const BUTTON_BORDER_WIDTH: f32 = 0.04;


// zvalue
pub const BOARD_Z_VALUE: i32 = -1;
pub const LEGAL_MOVE_Z_VALUE: i32 = 2;
pub const RING_MOVE_Z_VALUE: i32 = 40;
pub const TOKEN_Z_VALUE: i32 = 30;
pub const RING_Z_VALUE: i32 = 40;
pub const CURSOR_Z_VALUE: i32 = 50;
pub const RUN_Z_VALUE: i32 = 5;

// interaction
pub const SNAP_DISTANCE: f32 = 0.3;
pub const SNAP_DISTANCE_SQUARED: f32 = SNAP_DISTANCE * SNAP_DISTANCE;

// animation
pub const MOVE_ANIMATION_DURATION: f64 = 0.4;
pub const REMOVE_ANIMATION_DURATION: f64 = 0.2;
pub const FLIP_ANIMATION_DURATION: f64 = 0.2;

pub fn player_color(player: &Player) -> Color {
    match player {
        Player::White => WHITE_PLAYER_COLOR,
        Player::Black => BLACK_PLAYER_COLOR,
    }
}

pub fn opponent_color(player: &Player) -> Color {
    match player {
        Player::Black => WHITE_PLAYER_COLOR,
        Player::White => BLACK_PLAYER_COLOR,
    }
}
