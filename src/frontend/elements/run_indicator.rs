

use enumset::EnumSet;

use macroquad::prelude::*;

use crate::{
    core::coord::{HexCoord, Point},
    core::game::UiAction,
    frontend::{
        config::{RUN_INDICATOR_COLOR, RUN_INDICATOR_LINE_COLOR, RUN_INDICATOR_CIRCLE_SEGMENTS, RUN_INDICATOR_LINE_COLOR_HOVER, RUN_INDICATOR_COLOR_HOVER},
        element::{Element, Property},
        events::{Event, Message},
        mouse::mouse_leave_enter_event,
    },
};

use super::primitives::draw_run_indicator;

pub struct RunIndicator {
    corners: [Vec2; 4],
    dir: Vec2,
    perp: Vec2,
    coord: Option<HexCoord>,
    width: f32,
    height: f32,
    z_value: i32,
    hover_z_value: i32,
    default_z_value: i32,
    color: Color,
    line_color: Color,
    mouse_entered: bool,
    properties: EnumSet<Property>,
    is_visible: bool,
}

impl RunIndicator {
    pub fn from_segment_coords(
        coord0: HexCoord,
        coord1: HexCoord,
        height: f32,
        z_value: i32,
    ) -> Self {
        Self::from_segment_points(coord0.into(), coord1.into(), height, z_value)
    }

    pub fn set_coord(&mut self, coord: HexCoord) {
        self.coord = Some(coord);
    }

    pub fn from_segment_points(pt0: Point, pt1: Point, height: f32, z_value: i32) -> Self {
        let v1 = Vec2::from((pt0.0, pt0.1));
        let v2 = Vec2::from((pt1.0, pt1.1));
        let dir = (v2 - v1).normalize();
        let width = (v2 - v1).length();
        let perp = -dir.perp();

        let corners = [
            v1 + height / 2. * perp,
            v1 - height / 2. * perp,
            v2 - height / 2. * perp,
            v2 + height / 2. * perp,
        ];

        Self {
            corners,
            z_value,
            hover_z_value: 15,
            default_z_value: z_value,
            dir,
            perp,
            color: RUN_INDICATOR_COLOR,
            line_color: RUN_INDICATOR_LINE_COLOR,
            width,
            height,
            coord: None,
            mouse_entered: false,
            is_visible: true,
            properties: EnumSet::new(),
        }
    }

    pub fn add_property(&mut self, property: Property) {
        self.properties.insert(property);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    fn contains(&self, pos: Point) -> bool {
        let height = (self.corners[0] - self.corners[1]).length();
        let start = self.corners[0] - self.perp * height / 2.;
        let pt = vec2(pos.0, pos.1);

        let diff = pt - start;

        let proj = diff.dot(self.dir);
        if proj < 0. || proj > (self.corners[1] - self.corners[2]).length() {
            return false;
        }

        (diff - proj * self.dir).length_squared() <= (height / 2.).powi(2)
    }
}

impl Element for RunIndicator {
    fn render(&self) {
        draw_run_indicator(
            &self.corners,
            &self.perp,
            self.height,
            RUN_INDICATOR_CIRCLE_SEGMENTS,
            self.color,
            self.line_color,
        );
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        match message {
            Message::MouseEntered => {
                self.color = RUN_INDICATOR_COLOR_HOVER;
                self.line_color = RUN_INDICATOR_LINE_COLOR_HOVER;
                self.z_value = self.hover_z_value;
                self.mouse_entered = true;
                None
            }
            Message::MouseLeft => {
                self.color = RUN_INDICATOR_COLOR;
                self.line_color = RUN_INDICATOR_LINE_COLOR;
                self.z_value = self.default_z_value;
                None
            }
            Message::MouseClicked(_) => self.coord.map(|c| UiAction::ActionAtCoord(c)),
            _ => None,
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        match event {
            Event::Mouse(mouse_event) => {
                if self.properties.contains(Property::Hoverable) {
                    if let Some(e) = mouse_leave_enter_event(mouse_event, |pt| self.contains(*pt)) {
                        res.push(e);
                        return res;
                    };
                    if self.contains(mouse_event.pos) {
                        res.push(Message::MouseInside);
                    }
                }
                if self.properties.contains(Property::Clickable) {
                    if mouse_event.left_clicked
                        && self.contains(mouse_event.pos)
                        && self.coord.is_some()
                    {
                        res.push(Message::MouseClicked(self.coord.unwrap()));
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
