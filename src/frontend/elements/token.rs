

use super::{primitives::draw_ring_mesh, token_animation::{Animation, MoveAnimation, RemoveAnimation, FlipAnimation}};
use crate::{
    core::coord::{distance_squared, HexCoord, Point},
    core::{entities::Player, game::UiAction},
    frontend::{
        config::*,
        element::{Element, Property},
        events::{Event, Message},
        mouse::mouse_leave_enter_event,
    },
};
use enumset::{EnumSet};
use macroquad::prelude::*;

#[derive(Clone)]
pub enum TokenType {
    Ring(f32, f32),
    Marker(f32),
}

pub struct TokenBuilder {
    pos: Point,
    coord: Option<HexCoord>,
    pub token_type: Option<TokenType>,
    default_color: Color,
    select_color: Color,
    hover_color: Color,
    line_color: Color,
    properties: EnumSet<Property>,
    z_value: Option<i32>,
    alpha: f32,
}

impl TokenBuilder {
    pub fn new() -> Self {
        Self {
            pos: Point(0., 0.),
            coord: None,
            token_type: None,
            default_color: WHITE,
            select_color: SELECT_COLOR,
            line_color: DARK_BORDER_COLOR,
            hover_color: HOVER_COLOR,
            properties: EnumSet::new(),
            z_value: Some(TOKEN_Z_VALUE),
            alpha: 1.,
        }
    }

    pub fn ring(&mut self, player: Player) -> &mut Self {
        self.token_type = Some(TokenType::Ring(
            RING_OUTER_RADIUS,
            RING_INNER_RADIUS
        ));
        self.set_player(player);
        self
    }

    pub fn marker(&mut self, player: Player) -> &mut Self {
        self.token_type = Some(TokenType::Marker(MARKER_RADIUS));
        self.set_player(player);
        self
    }

    fn set_player(&mut self, player: Player) {
        match player {
            Player::Black => {
                self.default_color = BLACK_PLAYER_COLOR;
                self.line_color = LIGHT_BORDER_COLOR;
            }
            Player::White => {
                self.default_color = WHITE_PLAYER_COLOR;
                self.line_color = DARK_BORDER_COLOR;
            }
        }
    }

    pub fn remove_hover_color(&mut self) -> &mut Self {
        self.hover_color = REMOVE_COLOR;
        self
    }

    pub fn add_property(&mut self, property: Property) -> &mut Self {
        self.properties = self.properties | property;
        self
    }

    pub fn pos(&mut self, pos: Point) -> &mut Self {
        self.pos = pos;
        self
    }

    pub fn coord(&mut self, coord: HexCoord) -> &mut Self {
        self.coord = Some(coord);
        self.pos = Point::from(coord);
        self
    }

    pub fn z_value(&mut self, z_value: i32) -> &mut Self {
        self.z_value = Some(z_value);
        self
    }

    pub fn alpha(&mut self, alpha: f32) -> &mut Self {
        self.alpha = alpha;
        self
    }

    pub fn build_animated(&mut self) -> AnimatedToken {
        AnimatedToken::new(self.build(), None)
    }

    pub fn build(&mut self) -> Token {
        self.default_color.a = self.alpha;

        Token {
            pos: self.pos,
            coord: self.coord,
            shape_type: self.token_type.clone().unwrap(),
            color: self.default_color,
            default_color: self.default_color,
            hover_color: self.hover_color,
            line_color: self.line_color,
            select_color: self.select_color,
            properties: self.properties.clone(),
            z_value: self.z_value.unwrap(),
            mouse_entered: false,
            is_visible: true,
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pos: Point,
    coord: Option<HexCoord>,
    pub shape_type: TokenType,
    color: Color,
    line_color: Color,
    default_color: Color,
    hover_color: Color,
    select_color: Color,
    properties: EnumSet<Property>,
    z_value: i32,
    mouse_entered: bool,
    is_visible: bool
}

impl Token {
    pub fn new(
        pos: Point,
        coord: Option<HexCoord>,
        shape_type: TokenType,
        color: Color,
        line_color: Color,
        z_value: i32,
    ) -> Self {
        Token {
            pos,
            coord,
            shape_type,
            color,
            default_color: color,
            line_color,
            hover_color: HOVER_COLOR,
            select_color: SELECT_COLOR,
            properties: EnumSet::new(),
            z_value,
            mouse_entered: false,
            is_visible: true,
        }
    }

    pub fn draw(&self, color: Color, line_color: Color) {
        match self.shape_type {
            TokenType::Ring(radius_outer, radius_inner) => {
                draw_ring_mesh(self.pos.0, self.pos.1, radius_inner, radius_outer, color, RING_SEGMENTS);
                draw_ring_mesh(self.pos.0, self.pos.1, radius_outer, radius_outer + RING_BORDER_WIDTH, self.line_color, RING_SEGMENTS);
                draw_ring_mesh(self.pos.0, self.pos.1, radius_inner-RING_BORDER_WIDTH, radius_inner, self.line_color, RING_SEGMENTS);
            }
            TokenType::Marker(radius) => {
                draw_circle(self.pos.0, self.pos.1, radius, color);
                draw_circle_lines(
                    self.pos.0,
                    self.pos.1,
                    radius,
                    RING_BORDER_WIDTH,
                    line_color,
                );
            }
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn contains(&self, pos: Point) -> bool {
        match self.shape_type {
            TokenType::Marker(radius) => distance_squared(&self.pos, &pos) <= radius.powi(2),
            TokenType::Ring(outer, _) => distance_squared(&self.pos, &pos) <= outer.powi(2),
        }
    }

    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.pos = pos;
    }

    pub fn coord(&self) -> Option<HexCoord> {
        self.coord
    }

    pub fn has_property(&self, property: &Property) -> bool {
        self.properties.contains(*property)
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
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
}

impl Element for Token {
    fn render(&self) {
        if !self.is_visible {
            return;
        }
        if self.properties.contains(Property::Selected) {
            self.draw(self.select_color, self.line_color);
        } else {
            self.draw(self.color, self.line_color);
        }
    }

    fn update(&mut self, event: &Message) -> Option<UiAction> {
        let mut ret = None;
        match event {
            Message::MouseEntered => {
                self.color = self.hover_color;
                self.mouse_entered = true;
            }
            Message::MouseLeft => {
                if self.mouse_entered {
                    self.color = self.default_color;
                    self.mouse_entered = false;
                }
            }
            Message::MouseClicked(_) => {
                if self.coord.is_some() {
                    ret = Some(UiAction::ActionAtCoord(self.coord.unwrap()));
                }
            }
            Message::ElementMoved(pt) => self.pos = *pt,
            Message::ElementHide => self.is_visible = false,
            Message::ElementShow => self.is_visible = true,
            _ => (),
        }
        if self.properties.contains(Property::NoEventHandling) {
            None
        } else {
            ret
        }
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = vec![];
        if self.properties.contains(Property::NoEventHandling) {
            return res;
        }
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
                        && self.coord.is_some()
                        && self.contains(mouse_event.pos)
                    {
                        res.push(Message::MouseClicked(self.coord.unwrap()));
                    }
                }
                if self.properties.contains(Property::FollowMousePointer) {
                    let pos = mouse_event
                        .legal_move_coord
                        .map(Point::from)
                        .unwrap_or(mouse_event.pos);
                    res.push(Message::ElementMoved(pos));
                    let local_mouse_pos = mouse_position_local();
                    if local_mouse_pos.x < -0.9 || local_mouse_pos.x > 0.9 || local_mouse_pos.y < -0.9 || local_mouse_pos.y > 0.9 {
                        if self.is_visible {
                            res.push(Message::ElementHide);
                        }
                    } else {
                        if !self.is_visible {
                            res.push(Message::ElementShow);
                        }
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


pub struct AnimatedToken {
    token: Token,
    animation: Option<Box<dyn Animation>>,
}

impl AnimatedToken {
    pub fn new(token: Token, animation: Option<Box<dyn Animation>>) -> Self {
        AnimatedToken { token, animation }
    }
}

impl AnimatedToken {
    pub fn pos(&self) -> Point {
        self.token.pos()
    }

    pub fn coord(&self) -> Option<HexCoord> {
        self.token.coord()
    }

    pub fn set_pos(&mut self, pos: Point) {
        self.token.set_pos(pos);
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.token.contains(pos)
    }

    pub fn add_property(&mut self, property: Property) {
        self.token.add_property(property);
    }

    pub fn clear_properties(&mut self) {
        self.token.clear_properties();
    }
}

impl Element for AnimatedToken {
    fn render(&self) {
        self.token.render();
    }

    fn update(&mut self, message: &Message) -> Option<UiAction> {
        let res = self.token.update(message);
        if res.is_some() {
            return res;
        }

        match message {
            Message::FlipMarker(player, _coord) => {
                let start_color = player_color(player);
                self.token.set_color(start_color);
                self.animation = Some(Box::new(FlipAnimation::new(
                    self.token.shape_type.clone(),
                    start_color,
                    opponent_color(player),
                    2.,
                )));
                return Some(UiAction::AnimationInProgress);
            }
            Message::MoveRing(from, to) => {
                self.set_pos(Point::from(*from));
                self.animation = Some(MoveAnimation::new_box(Point::from(*from), Point::from(*to)));
                return Some(UiAction::AnimationInProgress);
            }
            Message::RemoveMarker(_coord) => {
                self.animation = Some(RemoveAnimation::new_box(self.token.shape_type.clone(), 1.2));
                return Some(UiAction::AnimationInProgress);
            }
            Message::Tick => {
                if self.animation.is_none() {
                    return Some(UiAction::AnimationFinished);
                }
                let animation = self.animation.as_mut().unwrap();

                animation.tick();
                animation.apply(&mut self.token);
                if self.animation.as_ref().unwrap().finished() {
                    self.animation = None;
                    return Some(UiAction::AnimationFinished);
                } else {
                    return Some(UiAction::AnimationInProgress);
                }
            }
            _ => (),
        }
        res
    }

    fn handle_event(&self, event: &Event) -> Vec<Message> {
        let mut res = self.token.handle_event(event);
        match event {
            Event::FlipMarker(player, coord) => {
                if self.coord() == Some(*coord) {
                    res.push(Message::FlipMarker(*player, *coord));
                }
            }
            Event::MoveRing(from, to) => {
                if self.coord() == Some(HexCoord::closest_coord_to_point(to).0) {
                    res.push(Message::MoveRing(*from, *to));
                }
            }
            Event::RemoveMarker(coord) => {
                if self.coord() == Some(*coord) {
                    res.push(Message::RemoveMarker(*coord));
                }
            }
            Event::Tick => {
                if self.animation.is_some() {
                    res.push(Message::Tick);
                }
            }
            _ => (),
        }
        res
    }

    fn z_value(&self) -> i32 {
        self.token.z_value()
    }
}
