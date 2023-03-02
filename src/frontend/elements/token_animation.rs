use std::f32::consts::PI;

use crate::core::coord::Point;
use macroquad::prelude::*;

use super::{token::{Token, TokenType, self}};
use crate::frontend::config::{MOVE_ANIMATION_DURATION, REMOVE_ANIMATION_DURATION, FLIP_ANIMATION_DURATION};


pub trait Animation {
    fn tick(&mut self);
    fn finished(&self) -> bool;
    fn apply(&self, marker: &mut Token);
}

#[derive(Clone)]
pub struct FlipAnimation {
    start_time: f64,
    duration: f64,
    start_color: Color,
    end_color: Color,
    current_color: Color,
    amplitude: f32,
    phase_shift: f32,
    value: f32,
    token_type: TokenType,
}

impl FlipAnimation {
    pub fn new(token_type: TokenType, start_color: Color, end_color: Color, expand_ratio: f32) -> Self {
        let phase_shift = (1. / expand_ratio).asin();
        FlipAnimation {
            start_time: get_time(),
            duration: FLIP_ANIMATION_DURATION,
            start_color,
            end_color,
            current_color: start_color,
            amplitude: expand_ratio,
            phase_shift,
            value: 1.,
            token_type: token_type
        }
    }

    pub fn new_box(token_type: TokenType, start_color: Color, end_color: Color, expand_ratio: f32) -> Box<Self> {
        Box::new(Self::new(token_type, start_color, end_color, expand_ratio))
    }
}

impl Animation for FlipAnimation {
    fn tick(&mut self) {
        let delta = (1. / self.duration * (get_time() - self.start_time)) as f32;
        self.current_color = Color::from_vec(
            self.start_color.to_vec()
                + delta * (self.end_color.to_vec() - self.start_color.to_vec()),
        );
        let t = (1. / self.duration * (get_time() - self.start_time)) as f32;
        let delta = self.phase_shift + t * (PI - 2.*self.phase_shift);
        self.value = self.amplitude * delta.sin();

        if self.value < 1. || self.finished() {
            self.value = 1.;
        }
    }

    fn apply(&self, marker: &mut Token) {
        marker.set_color(self.current_color);
        match self.token_type {
            TokenType::Ring(r1, r2) => {
                marker.shape_type = TokenType::Ring(self.value * r1, self.value * r2)
            }
            TokenType::Marker(r) => marker.shape_type = TokenType::Marker(self.value * r),
        }
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time > self.duration
    }
}

#[derive(Clone)]
pub struct RemoveAnimation {
    start_time: f64,
    duration: f64,
    amplitude: f32,
    phase_shift: f32,
    value: f32,
    token_type: TokenType,
}

impl RemoveAnimation {
    pub fn new(token_type: TokenType, expand_ratio: f32) -> Self {
        let phase_shift = (1. / expand_ratio).asin();

        RemoveAnimation {
            start_time: get_time(),
            duration: REMOVE_ANIMATION_DURATION,
            phase_shift,
            amplitude: expand_ratio,
            value: 1.,
            token_type,
        }
    }

    pub fn new_box(token_type: TokenType, expand_ratio: f32) -> Box<Self> {
        Box::new(Self::new(token_type, expand_ratio))
    }
}

impl Animation for RemoveAnimation {
    fn tick(&mut self) {
        if self.finished() {
            self.value = 0.;
        } else {
            let t = (1. / self.duration * (get_time() - self.start_time)) as f32;
            let delta = self.phase_shift + t * (PI - self.phase_shift);
            self.value = self.amplitude * delta.sin();
        }
    }

    fn apply(&self, marker: &mut Token) {
        match self.token_type {
            TokenType::Ring(r1, r2) => {
                marker.shape_type = TokenType::Ring(self.value * r1, self.value * r2)
            }
            TokenType::Marker(r) => marker.shape_type = TokenType::Marker(self.value * r),
        }
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time > self.duration
    }
}

#[derive(Clone)]
pub struct MoveAnimation {
    start_time: f64,
    duration: f64,
    start_pos: Point,
    end_pos: Point,
    current_pos: Point,
}

impl MoveAnimation {
    pub fn new(start_pos: Point, end_pos: Point) -> Self {
        MoveAnimation {
            start_time: get_time(),
            duration: MOVE_ANIMATION_DURATION,
            start_pos,
            end_pos,
            current_pos: start_pos,
        }
    }

    pub fn new_box(start_pos: Point, end_pos: Point) -> Box<Self> {
        Box::new(Self::new(start_pos, end_pos))
    }
}

impl Animation for MoveAnimation {
    fn tick(&mut self) {
        if self.finished() {
            self.current_pos = self.end_pos;
        } else {
            let delta = (1. / self.duration * (get_time() - self.start_time)) as f32;
            let delta = - ((PI*delta).cos() - 1.)/2.;
            self.current_pos = self.start_pos + (self.end_pos - self.start_pos) * delta;
        }
    }

    fn apply(&self, ring: &mut Token) {
        ring.set_pos(self.current_pos);
    }

    fn finished(&self) -> bool {
        get_time() - self.start_time >= self.duration
    }
}
