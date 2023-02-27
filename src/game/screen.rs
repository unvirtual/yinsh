use crate::game::{board::*, coord::norm_squared};
use macroquad::prelude::{*, camera::mouse};

use super::{coord::Coord, game::{Action, Command, Game, Phase}, entities::Player};

pub struct GameCanvas {
    grid: Vec<[(f32, f32); 2]>,
    legal_actions: Vec<Action>,
    pixel_width: f32,
    pixel_height: f32,
    width: f32,
    height: f32,
    mouse_coord: Option<(Coord, f32)>,
}

fn transform(x: f32, y:f32) -> (f32, f32) {
    (x*0.5*num::Float::sqrt(3.), y - 0.5*x)
}

impl GameCanvas {

    pub fn grid(radius: f32) -> Vec<[(f32, f32); 2]> {
        let dx: f32 = 0.5*(3. as f32).sqrt();
        let mut res = Vec::new();

        // diagonals
        for dy in [-0.5 as f32, 0.5 as f32] {
            let lambda: f32 = radius / (1.-dy.powi(2)).sqrt();
            let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

            for l in l0..=l1 {
                let l = l as f32;
                let det = (l.powi(2)*(dy.powi(2) - 1.) + radius.powi(2)).sqrt();
                if det <= 0. {
                    continue;
                }
                let mut mu1 = -l*dy - det;
                let mut mu2 = -l*dy + det;
                if l.abs() > radius {
                    mu1 = mu1.ceil();
                    mu2 = mu2.floor();
                } else {
                    mu1 = mu1.trunc();
                    mu2 = mu2.floor();
                }

                let vec = [(mu1*dx, l + mu1*dy), (mu2*dx, l + mu2*dy)];
                res.push(vec);
            }
        }

        // verticals
        let lambda: f32 = radius*2./3.*(3. as f32).sqrt();
        let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

        for l in l0..=l1 {
            let l = l as f32;
            let det = (4.*radius.powi(2) - 3.*l.powi(2)).sqrt();
            if det <= 0. {
                continue;
            }
            let mut mu1 = 0.5*(l - det);
            let mut mu2 = 0.5*(l + det);
            if l.abs() > radius {
                mu1 = mu1.ceil();
                mu2 = mu2.floor();
            } else {
                mu1 = mu1.trunc();
                mu2 = mu2.floor();
            }

            let vec = [(l*dx, -0.5*l + mu1), (l*dx, -0.5*l + mu2)];
            res.push(vec);
        }
        res
    }

    pub fn new(board: &Board, pixel_width: f32, pixel_height: f32, width: f32, height: f32) -> Self {
        let radius = board.get_radius();

        GameCanvas {
            grid: Self::grid(radius),
            width,
            height,
            pixel_width,
            pixel_height,
            legal_actions: Vec::new(),
            mouse_coord: None,
        }
    }

    pub fn to_pixel(&self, x: f32, y: f32) -> (f32, f32) {
        let h_ratio = self.pixel_width/ self.width;
        let w_ratio = self.pixel_height / self.height;

        (w_ratio*x + self.pixel_width/2., h_ratio*y + self.pixel_height/2.)

    }

    pub fn from_pixel(&self, x: (f32, f32)) -> (f32, f32) {
        let h_ratio = self.pixel_width/ self.width;
        let w_ratio = self.pixel_height / self.height;

        (1./w_ratio*(x.0 - self.pixel_width/2.), 1./h_ratio*(x.1 - self.pixel_height/2.))

    }


    pub fn update(&mut self, game: &mut Game) {
        let mouse_pos = self.from_pixel(mouse_position());
        let mouse_pos = (mouse_pos.0, -mouse_pos.1);
        self.mouse_coord = game.board.closest_field_to_xy(mouse_pos.0, mouse_pos.1);

        if self.legal_actions.len() == 0 {
            self.legal_actions = game.legal_moves();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some((coord, dist)) = self.mouse_coord {
                if dist < 0.04 {
                    if let Some(action) = self.legal_actions.iter().find(|a| a.coord() == coord) {
                        action.execute(game);
                        self.legal_actions = game.legal_moves();
                    }
                }
            }
        }

    }

    pub fn render(&self, game: &Game) {

        set_camera(&Camera2D {
            zoom: vec2(1. / self.width * 2., 1. / self.height * 2.),
            target: vec2(0., 0.),
            //rotation: 179.,
            ..Default::default()
        });

        for [p0, p1] in &self.grid {
            // let p0 = self.to_pixel(p0.0, p0.1);
            // let p1 = self.to_pixel(p1.0, p1.1);
            draw_line(p0.0, p0.1, p1.0, p1.1, 0.02, DARKGRAY);
        }

        for a in &self.legal_actions {
            let (x,y) = a.coord().to_xy();
            draw_circle(x, y, 0.05, BLUE);
        }

        for ring_coord in game.board.player_rings(Player::White) {
            let (x,y) = ring_coord.to_xy();
            // draw_circle_lines(x, y, 0.3, 0.2, WHITE);
            draw_poly_lines(x, y, 128, 0.3, 0., 0.2, WHITE);
        }

        for ring_coord in game.board.player_rings(Player::Black) {
            let (x,y) = ring_coord.to_xy();
            draw_poly_lines(x, y, 255, 0.3, 0., 0.2, BLACK);
            // draw_circle_lines(x, y, 0.3, 0.2, BLACK);
        }

        for marker_coord in game.board.player_markers(Player::White) {
            let (x,y) = marker_coord.to_xy();
            draw_circle(x, y, 0.2, WHITE);
        }

        for marker_coord in game.board.player_markers(Player::Black) {
            let (x,y) = marker_coord.to_xy();
            draw_circle(x, y, 0.2, BLACK);
        }

        if let Some((coord, dist)) = self.mouse_coord {
            if self.legal_actions.iter().find(|a| a.coord() == coord).is_some() && dist < 0.04 {
                let (x,y) = coord.to_xy();
                match game.current_phase {
                    Phase::PlaceRing => draw_circle_lines(x, y, 0.3,  0.2, LIME),
                    Phase::PlaceMarker => draw_circle(x, y,  0.3, LIME),
                    Phase::MoveRing(_) => draw_circle_lines(x, y,  0.3, 0.2, LIME),
                    Phase::RemoveRun => draw_circle(x, y,  0.3, RED),
                    Phase::RemoveRing => draw_circle(x, y,  0.3, RED),
                }
            }
        }

        let score_white_str = format!("White: {}", game.points_white);
        let score_black_str = format!("Black: {}", game.points_black);

        set_default_camera();
        draw_text(&score_white_str, 30., 100., 30., BLACK);
        draw_text(&score_black_str, 30., 150., 30., BLACK);


    }

}