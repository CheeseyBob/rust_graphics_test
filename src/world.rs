use std::collections::HashMap;
use rand::Rng;
use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;

#[derive(Copy, Clone)]
struct Direction { x: i8, y: i8 }

impl Direction {
    const NORTH: Direction = Direction::new(0, -1);
    const NORTHEAST: Direction = Direction::new(1, -1);
    const EAST: Direction = Direction::new(1, 0);
    const SOUTHEAST: Direction = Direction::new(1, 1);
    const SOUTH: Direction = Direction::new(0, 1);
    const SOUTHWEST: Direction = Direction::new(-1, 1);
    const WEST: Direction = Direction::new(-1, 0);
    const NORTHWEST: Direction = Direction::new(-1, -1);

    const fn new(x: i8, y: i8) -> Direction {
        Direction { x: Self::valid(x), y: Self::valid(y) }
    }

    const fn valid(coord: i8) -> i8 {
        match coord {
            1 | 0 | -1 => coord,
            _ => panic!("coordinates should be either 1, 0 or -1")
        }
    }

    fn random() -> Direction {
        match rand::thread_rng().gen_range(0..8) {
            0 => Direction::NORTH,
            1 => Direction::NORTHEAST,
            2 => Direction::EAST,
            3 => Direction::SOUTHEAST,
            4 => Direction::SOUTH,
            5 => Direction::SOUTHWEST,
            6 => Direction::WEST,
            7 => Direction::NORTHWEST,
            _ => panic!("generated index should be in range")
        }
    }
}

#[derive(Copy, Clone)]
struct Location { x: usize, y: usize }

impl Location {
    fn coordinates(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn plus(&self, direction: Direction) -> Location {
        Location {
            x: self.x.wrapping_add_signed(direction.x as isize),
            y: self.y.wrapping_add_signed(direction.y as isize),
        }
    }
}

pub struct Entity {
    location: Location,
}

impl Entity {
    fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.draw_pixel(self.location.x, self.location.y, Color::WHITE);
    }

    fn new(x: usize, y: usize) -> Entity {
        Entity {
            location: Location { x, y }
        }
    }

    fn set_location(&mut self, location: Location) {
        self.location = location;
    }

    fn step(&self) -> Action {
        Action::move_in_direction(self.location, Direction::random())
    }
}

enum Action {
    Wait,
    Move {
        from: Location,
        to: Location,
    }
}

impl Action {
    fn move_in_direction(location: Location, direction: Direction) -> Action {
        Action::Move {
            to: location.plus(direction),
            from: location,
        }
    }
}

pub struct World {
    entities: HashMap<(usize, usize), Entity>,
    width: usize,
    height: usize,
    rng_buffer: RngBuffer,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        World {
            entities: HashMap::new(),
            width,
            height,
            rng_buffer: RngBuffer::init_new(100_000, ())
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for entity in self.entities.values() {
            entity.draw(graphics);
        }
    }

    pub fn step(&mut self) {
        if self.entities.len() < 50 {
            match self.place_entity(Entity::new(50 + 10 * self.entities.len(), 100)) {
                Ok(_) => {}
                Err(_) => {}
            }
        }

        let actions: Vec<Action> = self.entities.values()
            .map(Entity::step)
            .collect();

        for action in actions {
            self.resolve_action(&action);
        }
    }

    fn resolve_action(&mut self, action: &Action) {
        match action {
            Action::Wait => {},
            Action::Move { from, to } => self.resolve_move(*from, *to),
        }
    }

    fn resolve_move(&mut self, from: Location, to: Location) {
        let mut entity = self.entities.remove(&from.coordinates()).expect("entity to move should be at this location");
        entity.set_location(to);
        match self.entities.insert(to.coordinates(), entity) {
            None => {}
            Some(_) => panic!("this location should be unoccupied")
        }
    }

    fn is_occupied(&self, location: &Location) -> bool {
        self.entities.get(&location.coordinates()).is_some()
    }

    fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        if self.is_occupied(&entity.location) { return Err(()) }

        match self.entities.insert(entity.location.coordinates(), entity) {
            Some(_) => panic!("location should not be occupied"),
            None => Ok(())
        }
    }
}