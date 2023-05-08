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

    pub fn fill_with<F>(&mut self, mut f: F)
        where F: FnMut() -> T {
        self.grid.fill_with(f);
    }

    // TODO - Change to accept Location instead of coordinates.
    pub fn get(&self, coordinates: (usize, usize)) -> &T {
        let index = self.index(coordinates);
        &self.grid[index]
    }

    // TODO - Change to accept Location instead of coordinates.
    pub fn get_mut(&mut self, coordinates: (usize, usize)) -> &mut T {
        let index = self.index(coordinates);
        &mut self.grid[index]
    }

    // TODO - Change to accept Location instead of coordinates.
    pub fn replace(&mut self, coordinates: (usize, usize), value: T) -> T {
        let index = self.index(coordinates);
        std::mem::replace(&mut self.grid[index], value)
    }

    // TODO - Change to accept Location instead of coordinates.
    fn index(&self, coordinates: (usize, usize)) -> usize {
        coordinates.0 + self.width * coordinates.1
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.grid.iter()
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
    pub fn x(&self) -> i8 {
        match self {
            Direction::North => 0,
            Direction::Northeast => 1,
            Direction::East => 1,
            Direction::Southeast => 1,
            Direction::South => 0,
            Direction::Southwest => -1,
            Direction::West => -1,
            Direction::Northwest => -1,
        }
    }

    pub fn y(&self) -> i8 {
        match self {
            Direction::North => -1,
            Direction::Northeast => -1,
            Direction::East => 0,
            Direction::Southeast => 1,
            Direction::South => 1,
            Direction::Southwest => 1,
            Direction::West => 0,
            Direction::Northwest => -1,
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
    x: usize,
    y: usize
}

impl Location {
    getter!(x: usize);
    getter!(y: usize);

    // TODO - Move this to Location impl.
    pub fn new<T>(x: usize, y: usize, grid: &Grid<T>) -> Location {
        Location {
            x: x % grid.width,
            y: y % grid.height,
        }
    }

    pub fn coordinates(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    // TODO - Move this to Location impl.
    pub fn plus<T>(&self, direction: Direction, grid: &Grid<T>) -> Location {
        Location {
            x: grid.width.wrapping_add_signed(direction.x() as isize).add(self.x) % grid.width,
            y: grid.height.wrapping_add_signed(direction.y() as isize).add(self.y) % grid.height,
        }
    }
}
