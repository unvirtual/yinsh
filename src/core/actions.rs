use enum_dispatch::enum_dispatch;

use crate::core::coord::*;
use super::command::*;
use super::{state::*, entities::*};

#[enum_dispatch(Command)]
#[derive(Debug, Clone)]
pub enum Action {
    PlaceRing,
    PlaceMarker,
    MoveRing,
    RemoveRun,
    RemoveRing,
}

#[derive(Debug, Clone)]
pub struct PlaceRing {
    pub coord: HexCoord,
}

#[derive(Debug, Clone)]
pub struct PlaceMarker {
    pub coord: HexCoord,
}

#[derive(Debug, Clone)]
pub struct MoveRing {
    pub from: HexCoord,
    pub to: HexCoord,
    pub player: Player,
}

#[derive(Debug, Clone)]
pub struct RemoveRun {
    pub run_idx: usize,
    pub run: Vec<HexCoord>,
    pub coord: HexCoord,
}

#[derive(Debug, Clone)]
pub struct RemoveRing {
    pub coord: HexCoord,
    pub player: Player,
}

impl Command for PlaceRing {
    fn is_legal(&self, state: &State) -> bool {
        state.at_phase(&Phase::PlaceRing) && state.board.free_board_field(&self.coord)
    }

    fn execute(&self, state: &mut State) {
        state.new_action();
        state.place_ring(&state.current_player.clone(), &self.coord);

        if state.board.rings().count() > 9 {
            state.set_phase(Phase::PlaceMarker);
        }

        state.next_player();
        state.history.push(Action::from(self.clone()));
    }

    fn undo(&self, state: &mut State) {
        state.new_action();
        state.remove_ring(&state.current_player.clone(), &self.coord);
        state.set_phase(Phase::PlaceRing);
        state.next_player();
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}

impl Command for PlaceMarker {
    fn is_legal(&self, state: &State) -> bool {
        state.at_phase(&Phase::PlaceMarker)
            && state.board.player_ring_at(&self.coord, &state.current_player)
    }

    fn execute(&self, state: &mut State) {
        state.new_action();
        state.place_marker(&state.current_player.clone(), &self.coord);
        state.set_phase(Phase::MoveRing(self.coord));
        state.history.push(Action::from(self.clone()));
    }

    fn undo(&self, state: &mut State) {
        state.new_action();
        state.place_ring(&state.current_player.clone(), &self.coord);
        state.set_phase(Phase::PlaceMarker);
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}

impl Command for MoveRing {
    fn is_legal(&self, state: &State) -> bool {
        if !state.at_phase(&Phase::MoveRing(self.from)) {
            return false;
        }
        return state
            .board
            .ring_targets(&self.from)
            .iter()
            .find(|&c| c == &self.to)
            .is_some();
    }

    fn execute(&self, state: &mut State) {
        state.new_action();
        state.move_ring(&state.current_player.clone(), &self.from, &self.to, false, true);
        state.flip_markers(&self.from, &self.to);

        state.compute_runs();

        if state.board.runs(&state.current_player).len() > 0 {
            state.set_phase(Phase::RemoveRun);
        } else if state.board.runs(&state.current_player.other()).len() > 0 {
            state.set_phase(Phase::RemoveRun);
            state.next_player();
        } else {
            state.set_phase(Phase::PlaceMarker);
            state.next_player();
        }

        state.history.push(Action::from(self.clone()));
    }

    fn undo(&self, state: &mut State) {
        state.new_action();
        state.current_player = self.player;
        state.move_ring(&state.current_player.clone(), &self.to, &self.from, true, false);

        state.flip_markers(&self.from, &self.to);

        state.set_phase(Phase::MoveRing(self.from));
        state.compute_runs();
    }

    fn coord(&self) -> HexCoord {
        self.to
    }
}

impl Command for RemoveRun {
    fn is_legal(&self, state: &State) -> bool {
        state.at_phase(&Phase::RemoveRun) && state.is_valid_run(&state.current_player, &self.run)
    }

    fn execute(&self, state: &mut State) {
        state.new_action();
        self.run.iter().for_each(|c| {
            state.remove_marker(&state.current_player.clone(), c);
        });

        state.compute_runs();
        state.set_phase(Phase::RemoveRing);
        state.history.push(Action::from(self.clone()));
    }

    fn undo(&self, state: &mut State) {
        state.new_action();
        state.set_phase(Phase::RemoveRun);
        self.run.iter().for_each(|c| {
            state.place_marker(&state.current_player.clone(), c);
        });
        state.compute_runs();
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}

impl Command for RemoveRing {
    fn is_legal(&self, state: &State) -> bool {
        state.at_phase(&Phase::RemoveRing)
            && state.board.player_ring_at(&self.coord, &state.current_player)
            && state.current_player == self.player
    }

    fn execute(&self, state: &mut State) {
        state.new_action();
        state.history.push(Action::from(self.clone()));

        state.remove_ring(&state.current_player.clone(), &self.coord);

        let current_player = state.current_player;
        state.inc_score(&current_player);

        if state.get_score(&current_player) == 3 {
            state.set_phase(Phase::PlayerWon(current_player));
            return;
        }

        if state.has_run(&state.current_player) {
            state.set_phase(Phase::RemoveRun);
            return;
        }

        state.next_player();

        if state.has_run(&state.current_player) {
            state.set_phase(Phase::RemoveRun);
        } else {
            state.set_phase(Phase::PlaceMarker);
        }
    }

    fn undo(&self, state: &mut State) {
        state.new_action();
        state.current_player = self.player;
        state.dec_score(&self.player);
        state.set_phase(Phase::RemoveRing);

        state.place_ring(&state.current_player.clone(), &self.coord);
    }

    fn coord(&self) -> HexCoord {
        self.coord
    }
}

#[cfg(test)]
mod test {
    use crate::core::board::Board;

    use super::*;

    #[test]
    fn test_place_ring() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;

        let c = HexCoord::new(2, 4);
        let action = PlaceRing { coord: c };

        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert_eq!(state.board.rings().count(), 1);
        assert!(state.board.player_ring_at(&c, &Player::White));
        assert_eq!(state.current_player, Player::Black);
        assert_eq!(state.current_phase, Phase::PlaceRing);

        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_place_ring_on_occupied_not_allowed() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;

        let c = HexCoord::new(2, 4);
        state.board
            .place_unchecked(&Piece::Marker(Player::White), &c);

        let action = PlaceRing { coord: c };

        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_place_ring_in_wrong_phase_not_allowed() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 4);
        let action = PlaceRing { coord: c };

        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_place_ring_undo() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;

        let occupied = HexCoord::new(-1, 1);
        state.board
            .place_unchecked(&Piece::Marker(Player::White), &occupied);

        let c = HexCoord::new(2, 4);
        let action = PlaceRing { coord: c };

        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert_eq!(state.board.rings().count(), 1);
        assert_eq!(state.board.markers().count(), 1);

        action.undo(&mut state);

        assert!(state.board.occupied(&c).is_none());
        assert!(state.board.occupied(&occupied).is_some());
        assert_eq!(state.current_phase, Phase::PlaceRing);
        assert_eq!(state.current_player, Player::White);

        assert_eq!(state.board.rings().count(), 0);
        assert_eq!(state.board.markers().count(), 1);
    }

    #[test]
    fn test_place_marker() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 4);

        let action = PlaceMarker { coord: c };
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);
        assert!(action.is_legal(&state));

        action.execute(&mut state);

        assert_eq!(state.board.markers().count(), 1);
        assert_eq!(state.board.rings().count(), 0);
        assert!(state.board.player_marker_at(&c, &Player::White));
        assert_eq!(state.current_player, Player::White);
        matches!(state.current_phase, Phase::MoveRing(_));
    }

    #[test]
    fn test_place_marker_without_ring() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 4);
        let action = PlaceMarker { coord: c };

        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_place_marker_wrong_player_ring() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 4);
        state.board.place_unchecked(&Piece::Ring(Player::Black), &c);
        let action = PlaceMarker { coord: c };
        assert!(!action.is_legal(&state));

        state.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = PlaceMarker { coord: c };
        assert!(action.is_legal(&state));
    }

    #[test]
    fn test_place_marker_in_wrong_phase_not_allowed() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::MoveRing(HexCoord::new(0, 0)));

        let c = HexCoord::new(2, 4);
        let action = PlaceMarker { coord: c };

        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_place_marker_undo() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 4);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = PlaceMarker { coord: c };
        assert!(action.is_legal(&state));

        assert!(action.is_legal(&state));
        action.execute(&mut state);
        action.undo(&mut state);

        assert!(state.board.player_ring_at(&c, &Player::White));
        assert_eq!(state.current_phase, Phase::PlaceMarker);
        assert_eq!(state.current_player, Player::White);
    }

    #[test]
    fn test_move_ring_without_run() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-1, -2);

        state.set_phase(Phase::MoveRing(from_coord));

        let to_coord = HexCoord::new(-1, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };

        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert_eq!(state.board.rings().count(), 1);
        assert!(state.board.player_ring_at(&to_coord, &Player::White));
        assert_eq!(state.current_player, Player::Black);
        assert_eq!(state.current_phase, Phase::PlaceMarker);
    }

    #[test]
    fn test_move_ring_in_wrong_phase() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-1, -2);

        state.set_phase(Phase::PlaceMarker);

        // not connected
        let to_coord = HexCoord::new(0, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_move_ring_to_illegal_field_not_allowed() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-1, -2);

        state.set_phase(Phase::MoveRing(from_coord));

        // not connected
        let to_coord = HexCoord::new(0, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(!action.is_legal(&state));

        // marker occupied
        let to_coord = HexCoord::new(-1, 4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        state.board
            .place_unchecked(&Piece::Marker(Player::Black), &to_coord);
        assert!(!action.is_legal(&state));

        // ring occupied
        let to_coord = HexCoord::new(-1, -4);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        state.board
            .place_unchecked(&Piece::Ring(Player::Black), &to_coord);
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_move_ring_flips_markers() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-2, 0);

        state.set_phase(Phase::MoveRing(from_coord));
        state.board
            .place_unchecked(&Piece::Marker(Player::Black), &HexCoord::new(0, 0));
        state.board
            .place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(1, 0));

        // not connected
        let to_coord = HexCoord::new(2, 0);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert!(state
            .board
            .player_marker_at(&HexCoord::new(0, 0), &Player::White));
        assert!(state
            .board
            .player_marker_at(&HexCoord::new(1, 0), &Player::Black));
        assert!(state.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_creates_run_from_placement() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-2, 0);

        state.set_phase(Phase::MoveRing(from_coord));
        for i in -2..=2 {
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(i, 0));
        }
        assert!(!state.has_run(&Player::White));

        // not connected
        let to_coord = HexCoord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert!(state.has_run(&Player::White));
        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::RemoveRun);
        assert!(state.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_creates_run_from_flip() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-2, -1);

        state.set_phase(Phase::MoveRing(from_coord));
        state.board
            .place_unchecked(&Piece::Marker(Player::Black), &HexCoord::new(-2, 0));
        for i in -1..=2 {
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(i, 0));
        }
        assert!(!state.has_run(&Player::White));

        // not connected
        let to_coord = HexCoord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert!(state.has_run(&Player::White));
        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::RemoveRun);
        assert!(state.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_move_ring_undo() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        let from_coord = HexCoord::new(-2, -1);

        state.set_phase(Phase::MoveRing(from_coord));
        state.board
            .place_unchecked(&Piece::Marker(Player::Black), &HexCoord::new(-2, 0));
        for i in -1..=2 {
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &HexCoord::new(i, 0));
        }
        assert!(!state.has_run(&Player::White));

        // not connected
        let to_coord = HexCoord::new(-2, 1);
        let action = MoveRing {
            player: Player::White,
            from: from_coord,
            to: to_coord,
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert!(state.has_run(&Player::White));
        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::RemoveRun);

        action.undo(&mut state);
        assert!(!state.has_run(&Player::White));
        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::MoveRing(from_coord));
        assert!(!state.board.player_ring_at(&to_coord, &Player::White));
    }

    #[test]
    fn test_remove_run() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRun);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        state.compute_runs();
        for c in run.iter() {
            assert!(state.board.player_marker_at(c, &Player::White));
        }
        assert!(state.has_run(&Player::White));
        assert_eq!(state.get_run(&Player::White, 0), Some(&run));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            coord: run[0],
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert!(!state.has_run(&Player::White));
        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::RemoveRing);
        for c in run.iter() {
            assert!(!state.board.player_marker_at(c, &Player::White));
        }
    }

    #[test]
    fn test_remove_run_wrong_phase() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        state.compute_runs();
        assert!(state.has_run(&Player::White));
        assert_eq!(state.get_run(&Player::White, 0), Some(&run));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            coord: run[0],
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_remove_run_illegal_index() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        state.compute_runs();
        assert!(state.has_run(&Player::White));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 2,
            coord: run[0],
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_remove_run_undo() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRun);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }
        // this is already set by previous step
        state.compute_runs();
        assert!(state.has_run(&Player::White));

        // not connected
        let action = RemoveRun {
            run: run.clone(),
            run_idx: 0,
            coord: run[0],
        };
        assert!(action.is_legal(&state));
        action.execute(&mut state);
        assert!(!state.has_run(&Player::White));

        action.undo(&mut state);
        assert!(state.has_run(&Player::White));
        assert_eq!(state.get_run(&Player::White, 0).unwrap(), &run);
        for c in run.iter() {
            assert!(state.board.player_marker_at(c, &Player::White));
        }
        assert_eq!(state.current_phase, Phase::RemoveRun);
        assert_eq!(state.current_player, Player::White);
    }

    #[test]
    fn test_remove_ring() {
        for player in [Player::White, Player::Black] {
        let mut state = State::new(Board::new());
            state.current_player = player;
            state.set_phase(Phase::RemoveRing);

            let c = HexCoord::new(2, 3);
            state.board.place_unchecked(&Piece::Ring(player), &c);

            // not connected
            let action = RemoveRing {
                coord: c.clone(),
                player,
            };
            match player {
                Player::White => assert_eq!(state.points_white, 0),
                Player::Black => assert_eq!(state.points_black, 0),
            }

            assert!(action.is_legal(&state));
            action.execute(&mut state);

            assert_eq!(state.current_player, player.other());
            assert_eq!(state.current_phase, Phase::PlaceMarker);
            assert!(!state.board.player_ring_at(&c, &player));
            match player {
                Player::White => assert_eq!(state.points_white, 1),
                Player::Black => assert_eq!(state.points_black, 1),
            }
        }
    }

    #[test]
    fn test_remove_ring_wrong_phase() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::PlaceMarker);

        let c = HexCoord::new(2, 3);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            coord: c.clone(),
            player: Player::White,
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_remove_ring_wrong_pos() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRing);

        let c = HexCoord::new(2, 3);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            coord: HexCoord::new(0, 0),
            player: Player::White,
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_remove_ring_wrong_player() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRing);

        let c = HexCoord::new(2, 3);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);

        // not connected
        let action = RemoveRing {
            coord: c.clone(),
            player: Player::Black,
        };
        assert!(!action.is_legal(&state));
    }

    #[test]
    fn test_remove_ring_player_runs() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRing);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::White), &c);
        }

        let c = HexCoord::new(2, 3);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = RemoveRing {
            coord: c.clone(),
            player: Player::White,
        };
        // this is already set by previous step
        state.compute_runs();
        assert!(state.has_run(&Player::White));
        assert_eq!(state.points_white, 0);

        // not connected
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert_eq!(state.current_player, Player::White);
        assert_eq!(state.current_phase, Phase::RemoveRun);
        assert_eq!(state.points_white, 1);
    }

    #[test]
    fn test_remove_ring_other_player_runs() {
        let mut state = State::new(Board::new());
        state.current_player = Player::White;
        state.set_phase(Phase::RemoveRing);

        let mut run = vec![];

        for i in -2..=2 {
            let c = HexCoord::new(i, 0);
            run.push(c);
            state.board
                .place_unchecked(&Piece::Marker(Player::Black), &c);
        }

        let c = HexCoord::new(2, 3);
        state.board.place_unchecked(&Piece::Ring(Player::White), &c);
        let action = RemoveRing {
            coord: c.clone(),
            player: Player::White,
        };
        // this is already set by previous step
        state.compute_runs();
        assert!(state.has_run(&Player::Black));
        assert_eq!(state.points_white, 0);

        // not connected
        assert!(action.is_legal(&state));
        action.execute(&mut state);

        assert_eq!(state.current_player, Player::Black);
        assert_eq!(state.current_phase, Phase::RemoveRun);
        assert_eq!(state.points_white, 1);
    }

    #[test]
    fn test_remove_ring_undo() {
        for player in [Player::White, Player::Black] {
        let mut state = State::new(Board::new());
            state.current_player = player;
            state.set_phase(Phase::RemoveRing);

            let c = HexCoord::new(2, 3);
            state.board.place_unchecked(&Piece::Ring(player), &c);

            // not connected
            let action = RemoveRing {
                coord: c.clone(),
                player,
            };
            match player {
                Player::White => assert_eq!(state.points_white, 0),
                Player::Black => assert_eq!(state.points_black, 0),
            }

            assert!(action.is_legal(&state));
            action.execute(&mut state);

            assert_eq!(state.current_player, player.other());
            assert_eq!(state.current_phase, Phase::PlaceMarker);
            assert!(!state.board.player_ring_at(&c, &player));
            match player {
                Player::White => assert_eq!(state.points_white, 1),
                Player::Black => assert_eq!(state.points_black, 1),
            }

            action.undo(&mut state);
            assert_eq!(state.current_player, player);
            assert_eq!(state.current_phase, Phase::RemoveRing);
            assert!(state.board.player_ring_at(&c, &player));
        }
    }
}