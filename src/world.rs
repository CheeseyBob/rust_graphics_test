use std::collections::HashMap;
use std::ptr::eq;
use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;
use crate::world::location::{Direction, Location};


mod location {
    use std::ops::Add;
    use crate::rng_buffer::RngBuffer;
    use crate::world::World;

    #[derive(Copy, Clone)]
    pub struct Direction { x: i8, y: i8 }

    impl Direction {
        pub const NORTH: Direction = Direction::new(0, -1);
        pub const NORTHEAST: Direction = Direction::new(1, -1);
        pub const EAST: Direction = Direction::new(1, 0);
        pub const SOUTHEAST: Direction = Direction::new(1, 1);
        pub const SOUTH: Direction = Direction::new(0, 1);
        pub const SOUTHWEST: Direction = Direction::new(-1, 1);
        pub const WEST: Direction = Direction::new(-1, 0);
        pub const NORTHWEST: Direction = Direction::new(-1, -1);

        pub const fn new(x: i8, y: i8) -> Direction {
            Direction { x: Self::valid(x), y: Self::valid(y) }
        }

        const fn valid(coord: i8) -> i8 {
            match coord {
                1 | 0 | -1 => coord,
                _ => panic!("coordinates should be either 1, 0 or -1")
            }
        }

        pub fn random(rng: &mut RngBuffer) -> Direction {
            match (rng.next() * 8.0) as usize {
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
    pub struct Location { x: usize, y: usize }

    impl Location {
        pub fn new(x: usize, y: usize, world: &World) -> Location {
            Location {
                x: x % world.width,
                y: y % world.height,
            }
        }

        pub fn coordinates(&self) -> (usize, usize) {
            (self.x, self.y)
        }

        pub fn plus(&self, direction: Direction, world: &World) -> Location {
            Location {
                x: world.width.wrapping_add_signed(direction.x as isize).add(self.x) % world.width,
                y: world.height.wrapping_add_signed(direction.y as isize).add(self.y) % world.height,
            }
        }

        pub fn x(&self) -> usize { self.x }

        pub fn y(&self) -> usize { self.y }
    }
}


pub struct Entity {
    location: Location,
    facing: Direction,
}

impl Entity {
    fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.draw_pixel(self.location.x(), self.location.y(), Color::WHITE);
    }

    fn new(x: usize, y: usize, world: &World, rng: &mut RngBuffer) -> Entity {
        Entity {
            location: Location::new(x, y, world),
            facing: Direction::random(rng),
        }
    }

    fn set_location(&mut self, location: Location) {
        self.location = location;
    }

    fn determine_action(&self, world: &World, rng: &mut RngBuffer) -> Action {
        match rng.next() {
            roll if roll < 0.05 => Action::turn(self.location, Direction::random(rng)),
            roll if roll < 0.95 => Action::move_in_direction(self.location, self.facing, world),
            _ => Action::wait(self.location),
        }
    }

    fn step(&self, rng: &mut RngBuffer) {

        // TODO ...

    }
}

enum Action {
    Wait {
        location: Location
    },
    Move {
        from: Location,
        to: Location,
    },
    Turn {
        location: Location,
        facing: Direction,
    },
}

impl Action {
    fn affected_locations(&self) -> Vec<&Location> {
        match self {
            Action::Wait { location } => vec![location],
            Action::Move { from, to } => vec![from, to],
            Action::Turn { location, facing: _ } => vec![location],
        }
    }

    fn resolve(&self, conflict_map: &HashMap<(usize, usize), Vec<&Action>>) -> Outcome {
        match self {
            Action::Wait { location } => Outcome::Wait { location: *location },
            Action::Move { from, to } => {
                let conflicts = conflict_map.get(&to.coordinates()).expect("conflict should be present at this location");
                for conflict in conflicts {
                    if !eq(*conflict, self) {
                        return Outcome::Wait { location: *from };
                    }
                }
                return Outcome::Move { from: *from, to: *to }
            }
            Action::Turn { location, facing } => Outcome::Turn { location: *location, facing: *facing },
        }
    }

    fn move_in_direction(location: Location, direction: Direction, world: &World) -> Action {
        Action::Move {
            to: location.plus(direction, world),
            from: location,
        }
    }

    fn turn(location: Location, facing: Direction) -> Action {
        Action::Turn { location, facing }
    }

    fn wait(location: Location) -> Action {
        Action::Wait { location }
    }
}

enum Outcome {
    Wait { location: Location },
    Move { from: Location, to: Location },
    Turn { location: Location, facing: Direction },
}

pub struct World {
    entities: HashMap<(usize, usize), Entity>,
    width: usize,
    height: usize,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        World {
            entities: HashMap::new(),
            width,
            height,
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for entity in self.entities.values() {
            entity.draw(graphics);
        }
    }

    pub fn load(&mut self, rng: &mut RngBuffer) {
        let mut unplaced_entity_count = 0;
        for _ in 0..5_000 {
            let x = (rng.generate_next() * self.width as f64) as usize;
            let y = (rng.next() * self.height as f64) as usize;
            let entity = Entity::new(x, y, &self, rng);
            if self.place_entity(entity).is_err() { unplaced_entity_count += 1 }
        }
        dbg!(unplaced_entity_count);
        /*
        for x in 100..200 {
            for y in 100..200 {
                let entity = Entity::new(x, y, &self, rng);
                self.place_entity(entity).expect("should be able to place here");
            }
        }
        */
    }

    pub fn step(&mut self, rng: &mut RngBuffer) {
        // Determine entity actions.
        let actions: Vec<Action> = self.entities.values()
            .map(|entity| entity.determine_action(&self, rng))
            .collect();

        // Find conflicting actions.
        let mut conflict_map: HashMap<(usize, usize), Vec<&Action>> = HashMap::with_capacity(actions.len());
        for action in &actions {
            for location in action.affected_locations() {
                let coordinates = location.coordinates();

                let mut conflict = conflict_map.remove(&coordinates)
                    .unwrap_or(Vec::new());
                conflict.push(&action);
                conflict_map.insert(coordinates, conflict);
            }
        }

        // Resolve actions while avoiding conflicts.
        let action_outcomes: Vec<Outcome> = actions.iter()
            .map(|action| action.resolve(&conflict_map))
            .collect();

        // Apply action outcomes.
        for outcome in action_outcomes {
            self.apply_action_outcome(outcome, rng);
        }
    }

    fn apply_action_outcome(&mut self, outcome: Outcome, rng: &mut RngBuffer) {
        match outcome {
            Outcome::Wait { location } => {
                self.resolve_wait(location, rng);
            }
            Outcome::Move { from, to } => {
                self.resolve_move(from, to, rng);
            }
            Outcome::Turn { location, facing } => {
                self.resolve_turn(location, facing, rng);
            }
        }
    }

    fn resolve_move(&mut self, from: Location, to: Location, rng: &mut RngBuffer) {
        let mut entity = self.entities.remove(&from.coordinates()).expect("entity should be at this location");
        entity.set_location(to);
        match self.entities.insert(to.coordinates(), entity) {
            None => {}
            Some(_) => panic!("this location should be unoccupied")
        }
    }

    fn resolve_turn(&mut self, location: Location, facing: Direction, rng: &mut RngBuffer) {
        let mut entity = self.entities.get_mut(&location.coordinates()).expect("entity should be at this location");
        entity.facing = facing;
    }

    fn resolve_wait(&mut self, location: Location, rng: &mut RngBuffer) {
        let mut entity = self.entities.get_mut(&location.coordinates()).expect("entity should be at this location");
        entity.step(rng);
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