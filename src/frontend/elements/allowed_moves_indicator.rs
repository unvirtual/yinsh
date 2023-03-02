use macroquad::prelude::*;

use crate::{
    core::coord::Point,
    core::game::UiAction,
    frontend::{
        element::{Element, Property},
        events::{Event, Message}, config::{LEGAL_MOVE_MARKER_COLOR},
    },
};

pub struct AllowedMovesIndicator {
    pos: Point,
    target: Point,
    properties: Property,
    z_value: i32,
    is_visible: bool,
}

impl AllowedMovesIndicator {
    pub fn new(pos: Point, target: Point, z_value: i32) -> Self {
        Self {
            pos,
            target,
            properties: Property::Nothing,
            z_value,
            is_visible: false,
        }
    }

    fn pos(&self) -> Point {
        self.pos
    }

    fn set_pos(&mut self, pos: Point) {
        self.pos = pos
    }

    pub fn add_property(&mut self, state: Property) {
        self.properties = state;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

}

impl Element for AllowedMovesIndicator {
    fn render(&self) {
        if self.is_visible {
            draw_line(
                self.pos.0,
                self.pos.1,
                self.target.0,
                self.target.1,
                0.1,
                LEGAL_MOVE_MARKER_COLOR,
            );
        }
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::ElementMoved(pos) => self.target = *pos,
            Message::ElementShow => self.is_visible = true,
            Message::ElementHide => self.is_visible = false,
            _ => (),
        }
        None
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if let Some(pos) = mouse_event.legal_move_coord.map(Point::from) {
                    if !self.is_visible {
                        res.push(Message::ElementShow);
                    }
                    res.push(Message::ElementMoved(pos));
                } else {
                    if self.is_visible {
                        res.push(Message::ElementHide);
                    }
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
