use crate::core::game::UiAction;
use super::events::*;
use enumset::EnumSetType;
use macroquad::prelude::*;

#[derive(EnumSetType, Hash)]
pub enum Property {
    Selected,
    FollowMousePointer,
    Hoverable,
    Animated,
    Clickable,
    NoEventHandling,
    Nothing
}

pub trait Element {
    fn render(&self);
    fn update(&mut self, message: &Message) -> Option<UiAction>;
    fn handle_event(&self, event: &Event) -> Vec<Message>;
    fn z_value(&self) -> i32;
}
