use std::{cmp, ops::{Add, Sub}};
use itertools::PeekingNext;
use num;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    N,
    NE,
    SE,
    S,
    SW,
    NW
}

impl Direction {
    pub fn dir_vec(&self) -> Coord {
        match *self {
            Direction::N => Coord(0, 1),
            Direction::NE => Coord(1, 1),
            Direction::SE => Coord(1, 0),
            Direction::S => Coord(0, -1),
            Direction::SW => Coord(-1, -1),
            Direction::NW => Coord(-1, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::N => Direction::S, 
            Direction::NE => Direction::SW,
            Direction::SE => Direction::NW,
            Direction::S => Direction::N,
            Direction::SW => Direction::NE,
            Direction::NW => Direction::SE,
        }
    }

    pub fn all() -> Vec<Direction> {
        vec![
            Direction::N,
            Direction::NE,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::NW,
        ]
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Coord(i8, i8);

impl Coord {
    pub fn new(x: i8, y: i8) -> Self {
        Coord(x,y)
    }

    pub fn cartesian_sq_norm(&self) -> f32 {
        num::pow(0.5*num::Float::sqrt(3.)*self.0 as f32, 2) + num::pow(self.1 as f32 - 0.5*self.0 as f32, 2)
    }

    pub fn sq_norm(&self) -> i8 {
        num::pow(self.0,2) + num::pow(self.1, 2)
    }

    pub fn neighbour(&self, direction: &Direction) -> Coord {
        *self + direction.dir_vec()
    }

    pub fn is_neighbour(&self, coord: &Coord) -> bool {
        let dir = self.dir_vec_to(coord);
        dir.map_or(false, |d| *self + d == *coord)
    }

    pub fn abs(&self) -> Coord {
        Coord(num::abs(self.0), num::abs(self.1))
    }

    pub fn is_dir_vec(&self) -> bool {
        let c = self.abs();
        c.0 <= 1 && c.1 <= 1
    }

    pub fn is_norm_dir_vec(&self) -> bool {
        self.is_dir_vec() && self.abs() == *self
    }

    pub fn dir_vec_to(&self, other: &Coord) -> Option<Coord> {
        if !self.connected_to(other) || self == other {
            return None;
        }
        let delta = *other - *self; //.abs()??;
        let div = cmp::max(num::abs(delta.0), num::abs(delta.1));

        Some(Coord(num::Integer::div_floor(&delta.0, &div), 
                   num::Integer::div_floor(&delta.1, &div))
        )
    }

    pub fn connected_to(&self, other: &Coord) -> bool {
        let Coord(x0,y0) = *self;
        let Coord(x1,y1) = *other;
        x0 == x1 || y0 == y1 || x0 - y0 == x1 - y1
    }

    pub fn line_iter(&self, dir: &Direction) -> LineIter {
        LineIter {
            current: self.clone(),
            dir: dir.dir_vec(),
        }
    }

    pub fn range_iter(&self, to: &Coord) -> Option<RangeIter> {
        self.dir_vec_to(to).and_then(|dir| 
            Some(RangeIter { 
                current: self.clone(), 
                to: to.clone(), 
                dir}
            )
        )
    }

    pub fn between_iter(&self, to: &Coord) -> Option<impl Iterator<Item=Coord>> {
        let to = to.clone();
        self.range_iter(&to).and_then(move |iter| 
            Some(
                iter.skip(1).take_while(move |x| *x != to)
            )
        )
    }
}

impl From<(i8, i8)> for Coord {
    fn from(t: (i8, i8)) -> Self {
        Coord::new(t.0, t.1)
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, other: Coord) -> Self {
        Coord(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, other: Coord) -> Self {
        Coord(self.0 - other.0, self.1 - other.1)
    }

}

pub struct LineIter {
    current: Coord,
    dir: Coord,
}

impl Iterator for LineIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(self.current);
        self.current = self.current + self.dir;
        ret
    }
}

// dir = current.dir_to(to) must hold!
pub struct RangeIter {
    current: Coord,
    to: Coord,
    dir: Coord,
}

impl Iterator for RangeIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.to + self.dir {
            return None;
        }
        let res = self.current;
        self.current = self.current + self.dir;
        Some(res)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn create_vec<T>(pos: T, dir: T, n: usize) -> Vec<Coord> 
    where 
        T: Into<Coord>  + Copy
    {
        let mut res = vec![];
        let mut curr = pos.into();
        for _ in 0..n {
            res.push(curr);
            curr = curr + dir.into();
        }
        res
    }

    #[test]
    fn test_line_iter() {
        let cases = 
            vec![(Coord::new(0,0), Direction::N, 5),
                 (Coord::new(2,2), Direction::NE, 7),
                 (Coord::new(-1,-3), Direction::SE, 3),
                 (Coord::new(1,-3), Direction::S, 1),
                 (Coord::new(-1,3), Direction::SW, 6),
                 (Coord::new(1,3), Direction::NW, 2),
            ];
        for (c, d, n) in cases {
            assert_eq!(c.line_iter(&d).take(n).collect::<Vec<_>>(), create_vec(c, d.dir_vec(), n));
        } 
    }

    #[test]
    fn test_between_iter() {
        assert!(Coord::new(1,1).between_iter(&Coord::new(1,1)).is_none());
        assert!(Coord::new(1,1).between_iter(&Coord::new(5,4)).is_none());

        let cases = 
            vec![(Coord::new(0,0), Coord::new(4,4), create_vec((1,1), (1,1), 3)),
                 (Coord::new(4,4), Coord::new(0,0), create_vec((3,3), (-1,-1), 3)),
                 (Coord::new(0,2), Coord::new(-3,-1), create_vec((-1,1), (-1,-1), 2)),
                 (Coord::new(-10,-3), Coord::new(12,-3), create_vec((-9,-3), (1,0), 21)),
                 (Coord::new(1,1), Coord::new(2,2), vec![]),
                 (Coord::new(1,-1), Coord::new(2,-1), vec![]),

            ];
        for (from, to, res) in cases {
            assert_eq!(from.between_iter(&to).unwrap().collect::<Vec<_>>(), res);
        } 
    }

    #[test]
    fn test_range_iter() {
        assert!(Coord::new(1,1).range_iter(&Coord::new(1,1)).is_none());
        assert!(Coord::new(1,1).range_iter(&Coord::new(5,4)).is_none());

        let cases = 
            vec![(Coord::new(0,0), Coord::new(4,4), create_vec((0,0), (1,1), 5)),
                 (Coord::new(4,4), Coord::new(0,0), create_vec((4,4), (-1,-1), 5)),
                 (Coord::new(0,2), Coord::new(-3,-1), create_vec((0,2), (-1,-1), 4)),
                 (Coord::new(-10,-3), Coord::new(12,-3), create_vec((-10,-3), (1,0), 23)),
                 (Coord::new(1,1), Coord::new(2,2), vec![Coord::new(1,1), Coord::new(2,2)]),
                 (Coord::new(1,-1), Coord::new(2,-1), vec![Coord::new(1,-1), Coord::new(2,-1)]),

            ];
        for (from, to, res) in cases {
            assert_eq!(from.range_iter(&to).unwrap().collect::<Vec<_>>(), res);
        } 
    }

}