use std::ops::Add;
use std::slice::Iter;
use crate::rng_buffer::RngBuffer;

pub struct Grid<T> {
    width: usize,
    height: usize,
    grid: Vec<T>,
}

impl<T> Grid<T> {
    getter!(width: usize);
    getter!(height: usize);

    pub fn new_filled_with<F>(mut f: F, width: usize, height: usize) -> Grid<T>
        where F: FnMut() -> T {
        Grid {
            grid: init_vec_with(f, width * height),
            width,
            height,
        }
    }

    pub fn fill_with<F: FnMut() -> T>(&mut self, mut f: F) {
        self.grid.fill_with(f);
    }

    pub fn get(&self, location: &Location) -> &T {
        &self.grid[location.index]
    }

    pub fn get_mut(&mut self, location: &Location) -> &mut T {
        &mut self.grid[location.index]
    }

    pub fn replace(&mut self, location: &Location, value: T) -> T {
        std::mem::replace(&mut self.grid[location.index], value)
    }

    pub fn add(&self, location: &Location, direction: &Direction) -> Location {
        let x = location.x + self.width.checked_add_signed(direction.x()).expect("adding +1/-1 to width should not overflow");
        let y = location.y + self.height.checked_add_signed(direction.y()).expect("adding +1/-1 to height should not overflow");
        Location::at(x, y, &self)
    }
}

fn init_vec_with<T, F>(mut f: F, capacity: usize) -> Vec<T>
    where F: FnMut() -> T {
    let mut vec: Vec<T> = Vec::with_capacity(capacity);
    vec.resize_with(capacity, f);
    return vec;
}

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

    pub fn random(rng: &mut RngBuffer) -> Direction {
        match (rng.next() * 8.0) as usize {
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

    pub fn at<T>(x: usize, y: usize, grid: &Grid<T>) -> Location {
        let x = x % grid.width;
        let y = y % grid.height;
        let index = x + grid.width * y;
        Location { x, y, index }
    }
}
