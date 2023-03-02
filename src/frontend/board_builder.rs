use super::config::*;
use super::events::Event;
use super::presenter::ElementId;
use macroquad::prelude::*;
use std::collections::{HashMap};

use crate::core::command::*;
use crate::{
    core::coord::{HexCoord, Point},
    core::{
        entities::Player,
        state::{Phase, State, StateChange},
    },
};

use super::elements::board::Board;
use super::{
    element::{Element, Property},
    elements::{
        allowed_moves_indicator::AllowedMovesIndicator, field_marker::FieldMarker,
        run_indicator::RunIndicator, token::TokenBuilder,
    },
    presenter::Presenter,
};

pub struct BoardBuilder {
    white_ring_slots: [Point; 3],
    black_ring_slots: [Point; 3],
    board: Board,
}

impl BoardBuilder {
    pub fn new(board_radius: f32, font: Font) -> Self {
        Self {
            white_ring_slots: Self::create_ring_slots(Point(-board_radius, -board_radius), 1.),
            black_ring_slots: Self::create_ring_slots(Point(board_radius, board_radius), -1.),
            board: Board::new(board_radius, font, -2),
        }
    }

    pub fn create_board_from_state(
        &mut self,
        state: &State,
        presenter: &mut Presenter,
        interactive: bool,
    ) {
        presenter.add_element(Box::new(self.board.clone()));

        self.create_static_elements(state, presenter);

        if interactive {
            presenter.schedule_event(Event::PlayerTurn(state.current_player, state.current_phase));
            self.trigger_animation_events(state, presenter);

            if state.current_player == Player::White {
                self.create_interactive_elements(state, presenter);
            }
        }
    }

    fn create_ring_slots(pos: Point, dist: f32) -> [Point; 3] {
        [
            pos,
            Point(pos.0 + dist, pos.1),
            Point(pos.0 + 2. * dist, pos.1),
        ]
    }

    fn ring_slots(&self, player: Player) -> &[Point; 3] {
        match player {
            Player::White => &self.white_ring_slots,
            Player::Black => &self.black_ring_slots,
        }
    }

    fn create_static_elements(&mut self, state: &State, presenter: &mut Presenter) {
        let runs = state.current_player_runs();

        for player in [Player::White, Player::Black] {
            add_won_rings(self.ring_slots(player), &player, &state, presenter);

            for c in state.board.player_rings(player) {
                add_ring_element(*c, player, &state, presenter);
            }

            for c in state.board.player_markers(player) {
                let marker_part_of_run = runs.iter().flatten().find(|&x| x == c).is_some();
                if !(state.current_phase == Phase::RemoveRun && marker_part_of_run) {
                    let token = TokenBuilder::new()
                        .marker(player)
                        .coord(*c)
                        .build_animated();
                    presenter.add_element(Box::new(token));
                }
            }
        }

        if state.current_phase == Phase::RemoveRun {
            add_run_indicators(&runs, state, presenter);
        }
    }

    fn trigger_animation_events(&mut self, state: &State, presenter: &mut Presenter) {
        for i in &state.last_state_change() {
            match i {
                StateChange::MarkerFlipped(coord) => {
                    if let Some(player) = state.board.belongs_to(&coord) {
                        presenter.schedule_event(Event::FlipMarker(player.other(), *coord));
                    }
                }
                StateChange::RingMoved(_player, from, to) => {
                    presenter.schedule_event(Event::MoveRing(Point::from(*from), Point::from(*to)));
                }
                StateChange::MarkerRemoved(player, coord) => {
                    let token = TokenBuilder::new()
                        .marker(*player)
                        .coord(*coord)
                        .build_animated();
                    presenter.add_element(Box::new(token));
                    presenter.schedule_event(Event::RemoveMarker(*coord));
                }
                StateChange::RingRemoved(player, coord) => {
                    if state.current_phase == Phase::PlaceMarker {
                        let score = state.get_score(player);
                        let slot_pt = self.ring_slots(*player)[score - 1];
                        // TODO THIS IS WRONG
                        let token = TokenBuilder::new()
                            .ring(*player)
                            .z_value(RING_Z_VALUE)
                            .coord(HexCoord::closest_coord_to_point(&slot_pt).0)
                            .build_animated();
                        presenter.add_element(Box::new(token));
                        presenter.schedule_event(Event::MoveRing(Point::from(*coord), slot_pt));
                    }
                }
                _ => (),
            }
        }
    }

    fn create_interactive_elements(&mut self, state: &State, presenter: &mut Presenter) {
        state.legal_moves().iter().for_each(|action| {
            let mut marker = FieldMarker::new(action.coord());
            if state.current_phase == Phase::PlaceRing {
                marker.set_visible(false);
            }
            presenter.add_element(Box::new(marker));
        });

        match state.current_phase {
            Phase::PlaceMarker => {
                add_marker_at_pointer(&Point(0., 0.), state, presenter);
            }
            Phase::PlaceRing => {
                add_ring_at_pointer(&Point(0., 0.), state, presenter);
            }
            Phase::MoveRing(from) => {
                add_ring_at_pointer(&Point(0., 0.), state, presenter);
                add_legal_moves_indicator(&from, presenter);
            }
            _ => (),
        }
    }
}

fn add_ring_element(c: HexCoord, player: Player, state: &State, presenter: &mut Presenter) {
    let mut builder = TokenBuilder::new();
    builder.ring(player).coord(c).z_value(RING_Z_VALUE);
    if player == Player::White
        && state.current_player == Player::White
        && state.current_phase == Phase::RemoveRing
    {
        builder.add_property(Property::Clickable);
        builder.add_property(Property::Hoverable);
        builder.remove_hover_color();
    }
    presenter.add_element(Box::new(builder.build_animated()));
}

fn add_run_indicators(r: &Vec<Vec<HexCoord>>, state: &State, presenter: &mut Presenter) {
    let mut added_marker: HashMap<HexCoord, ElementId> = HashMap::new();

    for (i, r) in r.iter().enumerate() {
        let mut run_indicator = Box::new(RunIndicator::from_segment_coords(
            r[0],
            *r.last().unwrap(),
            0.5,
            RUN_Z_VALUE + i as i32,
        ));
        run_indicator.set_coord(r[0]);
        if state.current_player == Player::White {
            run_indicator.add_property(Property::Hoverable);
            run_indicator.add_property(Property::Clickable);
        }
        let box_id = presenter.add_element(run_indicator);

        for c in r {
            if !added_marker.contains_key(&c) {
                let mut builder = TokenBuilder::new();
                if state.current_player == Player::White {
                    builder.add_property(Property::Hoverable);
                }
                let token = builder
                    .marker(state.current_player)
                    .coord(*c)
                    .add_property(Property::NoEventHandling)
                    .build();
                let marker_id = presenter.add_element_inactive(Box::new(token));
                added_marker.insert(*c, marker_id);
                presenter.add_subscriber(box_id, marker_id);
            } else {
                presenter.add_subscriber(box_id, *added_marker.get(&c).unwrap());
            }
        }
    }
}

fn add_ring_at_pointer(mouse_pos: &Point, state: &State, presenter: &mut Presenter) {
    let token = TokenBuilder::new()
        .ring(state.current_player)
        .pos(*mouse_pos)
        .z_value(CURSOR_Z_VALUE)
        .alpha(0.5)
        .add_property(Property::FollowMousePointer)
        .build();
    presenter.add_element(Box::new(token));
}

fn add_marker_at_pointer(mouse_pos: &Point, state: &State, presenter: &mut Presenter) {
    let token = TokenBuilder::new()
        .marker(state.current_player)
        .pos(*mouse_pos)
        .z_value(CURSOR_Z_VALUE)
        .add_property(Property::FollowMousePointer)
        .build();
    presenter.add_element(Box::new(token));
}

fn add_legal_moves_indicator(from: &HexCoord, presenter: &mut Presenter) {
    // ring at last position
    let token = TokenBuilder::new()
        .ring(Player::White)
        .coord(*from)
        .z_value(RING_Z_VALUE)
        .alpha(0.5)
        .add_property(Property::Selected)
        .build();
    presenter.add_element(Box::new(token));
    let element = Box::new(AllowedMovesIndicator::new(
        (*from).into(),
        (*from).into(),
        LEGAL_MOVE_Z_VALUE,
    ));
    presenter.add_element(element);
}

fn add_won_rings(
    ring_slots: &[Point; 3],
    player: &Player,
    state: &State,
    presenter: &mut Presenter,
) {
    let mut score = state.get_score(player);
    let player_scored = state
        .last_state_change
        .contains(&StateChange::PlayerScored(*player));

    if player_scored && !state.won_by().is_some() {
        score -= 1;
    }

    for i in 0..score {
        let token = TokenBuilder::new().ring(*player).pos(ring_slots[i]).build();
        presenter.add_element(Box::new(token));
    }
}
