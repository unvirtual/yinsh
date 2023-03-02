use macroquad::prelude::*;

use crate::{
    core::coord::{distance_squared, HexCoord, Point},
    core::game::UiAction,
    frontend::{
        element::{Element},
        events::{Event, Message}, config::{LEGAL_MOVE_MARKER_COLOR, LEGAL_MOVE_Z_VALUE, LEGAL_MOVE_MARKER_RADIUS, SNAP_DISTANCE},
    },
};

pub struct FieldMarker {
    pos: Point,
    z_value: i32,
    radius: f32,
    mouse_radius: f32,
    visible: bool,
    coord: HexCoord,
}

impl FieldMarker {
    pub fn new(coord: HexCoord) -> Self {
        Self {
            pos: Point::from(coord),
            coord,
            radius: LEGAL_MOVE_MARKER_RADIUS,
            mouse_radius: SNAP_DISTANCE,
            visible: true,
            z_value: LEGAL_MOVE_Z_VALUE,
        }
    }
}

impl FieldMarker {
    fn contains(&self, pos: Point) -> bool {
        distance_squared(&self.pos, &pos) <= self.mouse_radius.powi(2)
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Element for FieldMarker {
    fn render(&self) {
        if self.visible {
            draw_circle(self.pos.0, self.pos.1, self.radius, LEGAL_MOVE_MARKER_COLOR);
        }
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::MouseClicked(_) => Some(UiAction::ActionAtCoord(self.coord)),
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if mouse_event.left_clicked && self.contains(mouse_event.pos) {
                    res.push(Message::MouseClicked(self.coord));
                }
            }
            _ => (),
        }
        res
    }

    fn z_value(&self) -> i32 {
        self.z_value
    }
}
