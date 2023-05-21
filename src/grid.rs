use std::ops::Add;
use std::slice::Iter;
use parking_lot::{Mutex, MutexGuard, RawMutex};
use crate::rng_buffer;
use crate::world::World;

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North, Northeast, East, Southeast, South, Southwest, West, Northwest
}

impl Direction {
    pub fn x(&self) -> isize {
        match self {
            Direction::East | Direction::Northeast | Direction::Southeast => 1,
            Direction::West | Direction::Southwest | Direction::Northwest=> -1,
            Direction::North | Direction::South => 0,
        }
    }

    pub fn y(&self) -> isize {
        match self {
            Direction::South | Direction::Southeast | Direction::Southwest => 1,
            Direction::North | Direction::Northeast | Direction::Northwest => -1,
            Direction::East | Direction::West => 0,
        }
    }

    pub fn random() -> Direction {
        match (rng_buffer::next() * 8.0) as usize {
            0 => Direction::North,
            1 => Direction::Northeast,
            2 => Direction::East,
            3 => Direction::Southeast,
            4 => Direction::South,
            5 => Direction::Southwest,
            6 => Direction::West,
            7 => Direction::Northwest,
            _ => panic!("generated index should be in range")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Location {
    index: usize,
    x: usize,
    y: usize,
}

impl Location {
    getter!(x: usize);
    getter!(y: usize);
    getter!(index: usize);

    pub fn at(x: usize, y: usize, world: &World) -> Location {
        let x = x % world.width();
        let y = y % world.height();
        let index = x + world.width() * y;
        Location { x, y, index }
    }

    pub const fn zero() -> Location {
        Location { x: 0, y: 0, index: 0 }
    }
}
