use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use itertools::PeekingNext;

use crate::core::coord::*;
use crate::core::entities::*;

#[derive(Clone)]
pub struct Board {
    board_map: HashMap<HexCoord, Piece>,
    radius: f32,
}

impl Board {
    pub fn new() -> Self {
        Board {
            board_map: HashMap::new(),
            radius: 4.7,
        }
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn board_coords(&self) -> Vec<HexCoord> {
        let mut res = Vec::new();
        let radius = self.radius.ceil() as i8;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let c = HexCoord::new(dx, dy);
                if self.valid_coord(&c) {
                    res.push(c)
                }
            }
        }
        res
    }

    pub fn closest_field_to_xy(&self, x: f32, y: f32) -> Option<(HexCoord, f32)> {
        let closest = HexCoord::closest_coord_to_point(&Point(x, y));
        if self.valid_coord(&closest.0) {
            Some(closest)
        } else {
            None
        }
    }

    pub fn valid_coord(&self, coord: &HexCoord) -> bool {
        coord.cartesian_sq_norm() <= num::pow(self.radius, 2)
    }

    pub fn occupied(&self, coord: &HexCoord) -> Option<&Piece> {
        self.board_map.get(coord)
    }

    pub fn free_board_field(&self, coord: &HexCoord) -> bool {
        self.valid_coord(coord) && self.occupied(coord).is_none()
    }

    pub fn ring_at(&self, coord: &HexCoord) -> Option<&Piece> {
        self.occupied(coord).filter(|p| Piece::is_ring(p))
    }

    pub fn marker_at(&self, coord: &HexCoord) -> Option<&Piece> {
        self.occupied(coord).filter(|p| Piece::is_marker(p))
    }

    pub fn player_ring_at(&self, coord: &HexCoord, player: &Player) -> bool {
        self.ring_at(coord).map_or(false, |p| p.belongs_to(*player))
    }

    pub fn player_marker_at(&self, coord: &HexCoord, player: &Player) -> bool {
        self.marker_at(coord)
            .map_or(false, |p| p.belongs_to(*player))
    }

    fn filter_board<F>(&self, f: F) -> impl Iterator<Item = &HexCoord>
    where
        F: Fn(&HexCoord, &Piece) -> bool,
    {
        self.board_map
            .iter()
            .filter(move |(k, v)| f(k, v))
            .map(|(k, _)| k)
    }

    pub fn markers(&self) -> impl Iterator<Item = &HexCoord> {
        self.filter_board(|_, v| v.is_marker())
    }

    pub fn rings(&self) -> impl Iterator<Item = &HexCoord> {
        self.filter_board(|_, v| v.is_ring())
    }

    pub fn player_markers(&self, player: Player) -> impl Iterator<Item = &HexCoord> {
        self.filter_board(move |_, v| v.is_marker() && v.belongs_to(player))
    }

    pub fn player_rings(&self, player: Player) -> impl Iterator<Item = &HexCoord> {
        self.filter_board(move |_, v| v.is_ring() && v.belongs_to(player))
    }

    pub fn belongs_to(&self, coord: &HexCoord) -> Option<Player> {
        self.board_map.get(coord).map(|p| p.owner())
    }

    pub fn remove(&mut self, coord: &HexCoord) -> Option<Piece> {
        self.board_map.remove(coord)
    }

    pub fn place_unchecked(&mut self, piece: &Piece, coord: &HexCoord) -> Option<Piece> {
        self.board_map.insert(*coord, *piece)
    }

    pub fn place(&mut self, piece: &Piece, coord: &HexCoord) -> Option<Piece> {
        assert!(self.free_board_field(coord));
        self.place_unchecked(piece, coord)
    }

    fn ring_targets_in_dir(&self, from: &HexCoord, dir: &Direction) -> Vec<HexCoord> {
        let mut iter = from.line_iter(dir).skip(1).peekable();

        // take all empty fields along dir up to board boundary
        let mut ret: Vec<HexCoord> = iter
            .by_ref()
            .peeking_take_while(|c| self.occupied(c).is_none() && self.valid_coord(c))
            .collect();

        // return if the first non-empty field is a ring
        if iter
            .peeking_next(|c| self.ring_at(c).is_some() || !self.valid_coord(c))
            .is_some()
        {
            return ret;
        }

        // skip markers
        let mut iter = iter.skip_while(|c| self.marker_at(c).is_some()).peekable();

        // if the next non-marker field is empty and within board boundaries, add to result list
        if let Some(next) = iter.peeking_next(|c| self.occupied(c).is_none() && self.valid_coord(c))
        {
            ret.push(next);
        }

        ret
    }

    pub fn ring_targets(&self, from: &HexCoord) -> Vec<HexCoord> {
        Direction::all()
            .into_iter()
            .map(|dir| self.ring_targets_in_dir(from, &dir))
            .flatten()
            .collect()
    }

    fn marker_run_in_dir(
        &self,
        player: &Player,
        coord: &HexCoord,
        dir: &Direction,
    ) -> Vec<HexCoord> {
        if self.marker_at(coord).is_none() {
            panic!("marker_run_in_dir: no marker at coord {:?}", coord);
        }

        let in_dir = coord
            .line_iter(dir)
            .take_while(|c| self.player_marker_at(c, player));

        let oppo_dir = coord
            .line_iter(&dir.opposite())
            .skip(1)
            .take_while(|c| self.player_marker_at(c, player));

        oppo_dir
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .chain(in_dir)
            .collect()
    }

    pub fn runs(&self, player: &Player) -> Vec<Vec<HexCoord>> {
        let mut ret: Vec<Vec<HexCoord>> = vec![];

        for dir in [Direction::N, Direction::NE, Direction::SE].iter() {
            let mut cache: HashSet<HexCoord> = HashSet::new();
            for mcoord in self.player_markers(*player) {
                if cache.contains(mcoord) {
                    continue;
                }
                let res = self.marker_run_in_dir(player, mcoord, dir);
                cache.extend(&res);
                ret.extend(res.as_slice().windows(5).map(|x| x.to_vec()));
            }
        }
        ret
    }

    pub fn n_connected_markers(&self, player: &Player, length: usize) -> usize {
        let mut result = 0;
        for dir in [Direction::N, Direction::NE, Direction::SE].iter() {
            let mut cache: HashSet<HexCoord> = HashSet::new();
            for mcoord in self.player_markers(*player) {
                if cache.contains(mcoord) {
                    continue;
                }
                let res = self.marker_run_in_dir(player, mcoord, dir);
                cache.extend(&res);
                if res.len() == length {
                    result += 1;
                }
            }
        }
        result
    }

    pub fn flip_marker(&mut self, coord: &HexCoord) -> bool {
        if self.marker_at(coord).is_some() {
            // safe, guarded by marker_at()
            let marker = self.remove(coord).unwrap().flip().unwrap();
            self.place_unchecked(&marker, coord);
            return true;
        }
        false
    }

    pub fn flip_between(&mut self, start: &HexCoord, end: &HexCoord) -> Vec<HexCoord> {
        let mut res = vec![];
        if let Some(iter) = start.between_iter(end) {
            iter.for_each(|c| {
                if self.flip_marker(&c) {
                    res.push(c);
                }
            });
        }
        res
    }

    pub fn clear(&mut self) {
        self.board_map.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ring_target() {
        let mut board = Board::new();
        board.place_unchecked(&Piece::Marker(Player::White), &(1, 0).into());
        board.place_unchecked(&Piece::Marker(Player::White), &(2, 0).into());
        board.place_unchecked(&Piece::Ring(Player::White), &(-1, 3).into());
        board.place_unchecked(&Piece::Marker(Player::White), &(-1, -1).into());
        board.place_unchecked(&Piece::Marker(Player::White), &(5, 4).into());
        board.place_unchecked(&Piece::Ring(Player::White), &(-1, -2).into());

        // free, 2 markers, free, boundary
        assert_eq!(
            board.ring_targets_in_dir(&(-1, 0).into(), &Direction::SE),
            vec![HexCoord::new(0, 0), HexCoord::new(3, 0)]
        );

        // free until boundary
        assert_eq!(
            board.ring_targets_in_dir(&(-1, 0).into(), &Direction::NE),
            vec![
                HexCoord::new(0, 1),
                HexCoord::new(1, 2),
                HexCoord::new(2, 3),
                HexCoord::new(3, 4),
                HexCoord::new(4, 5)
            ]
        );

        // 2 free until ring
        assert_eq!(
            board.ring_targets_in_dir(&(-1, 0).into(), &Direction::N),
            vec![HexCoord::new(-1, 1), HexCoord::new(-1, 2)]
        );

        // marker ring free
        assert_eq!(
            board.ring_targets_in_dir(&(-1, 0).into(), &Direction::S),
            vec![]
        );

        // ring
        assert_eq!(
            board.ring_targets_in_dir(&(-1, 2).into(), &Direction::N),
            vec![]
        );

        // marker at boundary
        assert_eq!(
            board.ring_targets_in_dir(&(3, 4).into(), &Direction::SE),
            vec![HexCoord::new(4, 4)]
        );
    }

    #[test]
    fn flip_marker_test() {
        let mut board = Board::new();
        // should not panic, flip marker on empty or ring is noop
        let p = Player::White;

        board.flip_marker(&HexCoord::new(0, 0));
        board.place_unchecked(&Piece::Ring(p), &HexCoord::new(0, 0));
        board.flip_marker(&HexCoord::new(0, 0));

        let c = HexCoord::new(2, 3);
        board.place_unchecked(&Piece::Marker(p), &c);
        board.flip_marker(&c);

        assert!(board
            .marker_at(&c)
            .map(|m| m.belongs_to(p.other()))
            .unwrap())
    }

    fn markers_on_board(board: &Board, pieces: Vec<((i8, i8), Player)>) -> bool {
        pieces
            .iter()
            .all(|(t, p)| board.player_marker_at(&(*t).into(), p))
    }

    #[test]
    fn flip_between_test() {
        let markers: Vec<(i8, i8)> = vec![(-1, 0), (1, 0), (3, 0), (4, 0)];
        let mut board = Board::new();

        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });
        board.flip_between(&HexCoord::new(-2, 0), &HexCoord::new(5, 0));
        assert_eq!(board.player_markers(Player::White).count(), 0);
        assert_eq!(board.player_markers(Player::Black).count(), markers.len());

        board.clear();
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });
        board.flip_between(&HexCoord::new(-1, 0), &HexCoord::new(4, 0));
        assert!(markers_on_board(
            &board,
            vec![
                ((-1, 0), Player::White),
                ((1, 0), Player::Black),
                ((3, 0), Player::Black),
                ((4, 0), Player::White)
            ]
        ));

        board.clear();
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });
        board.flip_between(&HexCoord::new(1, 0), &HexCoord::new(2, 0));
        assert!(markers_on_board(&board, vec![]));

        assert_eq!(board.markers().count(), markers.len());
        assert_eq!(board.rings().count(), 0);
    }

    #[test]
    fn find_single_run() {
        let mut board = Board::new();

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(-1, 0), (0, 0), (1, 0), (2, 0), (3, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 1);
        assert_eq!(
            runs_white[0],
            markers
                .iter()
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(runs_black.len(), 0);
    }

    #[test]
    fn find_overlapping_runs() {
        let mut board = Board::new();

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(-1, 0), (0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 2);
        assert_eq!(
            runs_white[0],
            markers
                .iter()
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            runs_white[1],
            markers
                .iter()
                .skip(1)
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(runs_black.len(), 0);

        board.clear();
        let markers: Vec<(i8, i8)> = vec![(0, -2), (0, -1), (0, 0), (0, 1), (0, 2), (0, 3), (0, 4)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 3);
        assert_eq!(
            runs_white[0],
            markers
                .iter()
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            runs_white[1],
            markers
                .iter()
                .skip(1)
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            runs_white[2],
            markers
                .iter()
                .skip(2)
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(runs_black.len(), 0);

        board.clear();
        let markers: Vec<(i8, i8)> = vec![(-1, -1), (0, 0), (1, 1), (2, 2), (3, 3), (4, 4)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 2);
        assert_eq!(
            runs_white[0],
            markers
                .iter()
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            runs_white[1],
            markers
                .iter()
                .skip(1)
                .take(5)
                .map(|x| HexCoord::from(*x))
                .collect::<Vec<_>>()
        );
        assert_eq!(runs_black.len(), 0);
    }

    #[test]
    fn find_no_runs() {
        let mut board = Board::new();

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(-1, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::Black), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(1, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::Black), &HexCoord::from(*c));
        });
        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(0, 0), (3, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::Black), &HexCoord::from(*c));
        });
        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        let markers: Vec<(i8, i8)> = vec![(2, 0)];
        markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });
        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 0);

        board.flip_marker(&HexCoord::new(2, 0));
        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 0);
        assert_eq!(runs_black.len(), 1);
    }

    #[test]
    fn find_multiple_runs() {
        let mut board = Board::new();

        let black_markers: Vec<(i8, i8)> = vec![
            (-1, 0),
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (-2, -1),
            (-1, 0),
            (0, 1),
            (1, 2),
            (2, 3),
        ];
        let white_markers: Vec<(i8, i8)> = vec![(-3, -2), (-3, -1), (-3, 0), (-3, 1), (-3, 2)];
        black_markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::Black), &HexCoord::from(*c));
        });
        white_markers.iter().for_each(|c| {
            board.place_unchecked(&Piece::Marker(Player::White), &HexCoord::from(*c));
        });

        let runs_white = board.runs(&Player::White);
        let runs_black = board.runs(&Player::Black);
        assert_eq!(runs_white.len(), 1);
        assert_eq!(runs_black.len(), 2);
    }
}
