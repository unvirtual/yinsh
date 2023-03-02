use macroquad::prelude::*;

use macroquad::models::Vertex;

use crate::{
    core::coord::{HexCoord, HexCoordF, Point},
    core::{entities::Player, game::UiAction, state::Phase},
    frontend::{
        config::{GRID_LINE_COLOR, GRID_LNE_WIDTH},
        element::{Element, Property},
        events::{Event, Message},
    },
};

use super::primitives::{build_grid_hull_mesh, build_grid_lines};

#[derive(Clone)]
pub struct Board {
    z_value: i32,
    radius: f32,
    grid_lines: Vec<[Point; 2]>,
    grid_border_vertices: Vec<Vertex>,
    grid_border_indices: Vec<u16>,
    status_text: String,
    font: Font,
}

impl Board {
    pub fn new(radius: f32, font: Font, z_value: i32) -> Self {
        let grid_lines = build_grid_lines(radius);
        let (grid_border_vertices, grid_border_indices) = build_grid_hull_mesh(&grid_lines);
        Self {
            radius,
            z_value,
            grid_lines,
            grid_border_vertices,
            grid_border_indices,
            status_text: String::new(),
            font,
        }
    }

    fn draw_grid(&self) {
        draw_mesh(&Mesh {
            vertices: self.grid_border_vertices.clone(),
            indices: self.grid_border_indices.clone(),
            texture: None,
        });
        for [p0, p1] in &self.grid_lines {
            draw_line(p0.0, p0.1, p1.0, p1.1, GRID_LNE_WIDTH, GRID_LINE_COLOR);
        }
    }

    fn contains(&self, _pos: Point) -> bool {
        true
    }
}

impl Element for Board {
    fn render(&self) {
        self.draw_grid();
        let (font_size, font_scale, font_aspect) = camera_font_scale(0.3);
        let text_params = TextParams {
            font_size,
            font_scale: -font_scale,
            font_scale_aspect: -font_aspect,
            color: BLACK,
            font: self.font,
            ..Default::default()
        };
        let center = get_text_center(&self.status_text, Some(self.font), font_size, font_scale, 0.);
        draw_text_ex(
            &self.status_text,
            -center.x,
            self.radius + 0.5 - center.y,
            text_params,
        );
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            //Message::MouseClicked(_) => Some(UiAction::Undo),
            Message::PlayerTurn(player, phase) => {
                match (player, phase) {
                    (Player::White, Phase::PlaceRing) => {
                        self.status_text = format!("Your turn, place a ring!")
                    }
                    (Player::White, Phase::PlaceMarker) => {
                        self.status_text = format!("Your turn, place a marker!")
                    }
                    (Player::White, Phase::RemoveRun) => {
                        self.status_text = format!("You created a run, pick one!")
                    }
                    (Player::White, Phase::RemoveRing) => {
                        self.status_text = format!("Your turn, remove a ring!")
                    }
                    (Player::White, Phase::MoveRing(_)) => {
                        self.status_text = format!("Your turn, place ring!")
                    }
                    (Player::White, _) => self.status_text = format!("Your turn!"),
                    (Player::Black, _) => self.status_text = format!("Yinsh bot thinks ..."),
                };
                None
            }
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if mouse_event.right_clicked && self.contains(mouse_event.pos) {
                    res.push(Message::MouseClicked(
                        HexCoord::closest_coord_to_point(&mouse_event.pos).0,
                    ));
                }
            }
            Event::PlayerTurn(player, phase) => res.push(Message::PlayerTurn(*player, *phase)),
            _ => (),
        }
        res
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}
