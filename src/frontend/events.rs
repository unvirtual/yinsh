use crate::{core::coord::{Point, HexCoord}, core::{entities::Player, state::Phase}};
use super::mouse::MouseEvent;

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    MouseEntered,
    MouseLeft,
    MouseInside,
    ElementMoved(Point),
    ElementShow,
    ElementHide,
    MouseClicked(HexCoord),
    Tick,
    FlipMarker(Player, HexCoord),
    MoveRing(Point, Point),
    RemoveMarker(HexCoord),
    PlayerTurn(Player, Phase),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    FlipMarker(Player, HexCoord),
    RemoveMarker(HexCoord),
    RemoveRing(HexCoord),
    MoveRing(Point, Point),
    PlaceRing(Player, HexCoord),
    PlayerTurn(Player, Phase),
    Tick,
}
