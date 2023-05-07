use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;
use crate::world::location::{Direction, Location};


mod location {
    use std::ops::Add;
    use crate::rng_buffer::RngBuffer;
    use crate::world::World;

    #[derive(Copy, Clone)]
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
                x: world.width.wrapping_add_signed(direction.x() as isize).add(self.x) % world.width,
                y: world.height.wrapping_add_signed(direction.y() as isize).add(self.y) % world.height,
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
            roll if roll < 0.95 => Action::move_in_direction(self.location, self.facing),
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
        direction: Direction,
    },
    Turn {
        location: Location,
        facing: Direction,
    },
}

impl Action {
    fn conflicts(&self) -> Conflict {
        match self {
            Action::Wait { .. } => Conflict::none(),
            Action::Move { from, direction } => Conflict::from_direction(*direction),
            Action::Turn { .. } => Conflict::none(),
        }
    }

    fn resolve(&self, world: &World, entity: &Entity) -> Outcome {
        match self {
            Action::Wait { location } => Outcome::Wait { location: *location },
            Action::Move { from, direction } => {
                let target_location = entity.location.plus(*direction, world);
                match world.get_entity(target_location) {
                    Some(_) => Outcome::Blocked,
                    None => Outcome::Move { from: *from, direction: *direction },
                }
            }
            Action::Turn { location, facing } => Outcome::Turn { location: *location, facing: *facing },
        }
    }

    fn move_in_direction(location: Location, direction: Direction) -> Action {
        Action::Move {
            from: location,
            direction,
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
    Blocked,
    Wait { location: Location },
    Move { from: Location, direction: Direction },
    Turn { location: Location, facing: Direction },
}

pub struct World {
    entity_grid: Vec<Option<Entity>>,
    width: usize,
    height: usize,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        World {
            entity_grid: init_vec_with(|| None,width * height),
            width,
            height,
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for space in &self.entity_grid {
            match space {
                Some(entity) => entity.draw(graphics),
                None => {}
            }
        }
    }

    pub fn load(&mut self, rng: &mut RngBuffer) {
        let mut count = 0;
        while count < 5_000 {
            let x = (rng.generate_next() * self.width as f64) as usize;
            let y = (rng.next() * self.height as f64) as usize;
            let entity = Entity::new(x, y, &self, rng);
            if self.place_entity(entity).is_ok() {
                count += 1;
            }
        }
    }

    pub fn step(&mut self, rng: &mut RngBuffer) {
        let space_count = self.width * self.height;

        // Determine entity actions.
        let mut actions: Vec<Option<Action>> = init_vec_with(|| None, space_count);
        for i in 0..space_count {
            actions[i] = match &self.entity_grid[i] {
                Some(entity) => Some(entity.determine_action(&self, rng)),
                None => None,
            };
        }

        // Find conflicting actions.
        let mut conflicts: Vec<Conflict> = init_vec_with(Conflict::none, space_count);
        for x in 0..self.width {
            for y in 0..self.height {
                match &actions[x + y * self.width] {
                    None => {}
                    Some(action) => {
                        let conflict = action.conflicts();
                        let location = Location::new(x, y, &self);

                        let north = location.plus(Direction::North, &self);
                        let northeast = location.plus(Direction::Northeast, &self);
                        let east = location.plus(Direction::East, &self);
                        let southeast = location.plus(Direction::Southeast, &self);
                        let south = location.plus(Direction::South, &self);
                        let southwest = location.plus(Direction::Southwest, &self);
                        let west = location.plus(Direction::West, &self);
                        let northwest = location.plus(Direction::Northwest, &self);

                        conflicts[self.buffer_index(north)].south = conflict.north;
                        conflicts[self.buffer_index(northeast)].southwest = conflict.northeast;
                        conflicts[self.buffer_index(east)].west = conflict.east;
                        conflicts[self.buffer_index(southeast)].northwest = conflict.southeast;
                        conflicts[self.buffer_index(south)].north = conflict.south;
                        conflicts[self.buffer_index(southwest)].northeast = conflict.southwest;
                        conflicts[self.buffer_index(west)].east = conflict.west;
                        conflicts[self.buffer_index(northwest)].southeast = conflict.northwest;
                    }
                }
            }
        }

        // Resolve conflicts.
        let mut outcomes: Vec<Option<Outcome>> = init_vec_with(|| None, space_count);
        for x in 0..self.width {
            for y in 0..self.height {
                match &actions[x + y * self.width] {
                    None => {}
                    Some(action) => {
                        let location = Location::new(x, y, &self);
                        for direction in action.conflicts().directions() {
                            let conflict_direction = location.plus(direction, &self);
                            let index = self.buffer_index(conflict_direction);
                            if conflicts[index].is_conflicted() {
                                outcomes[x + y * self.width] = Some(Outcome::Blocked);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // Resolve actions.
        for i in 0..space_count {
            if outcomes[i].is_some() {
                continue;
            }
            outcomes[i] = match &actions[i] {
                None => None,
                Some(action) => {
                    let entity = self.entity_grid[i].as_ref().expect("entity should be at this location");
                    Some(action.resolve(&self, entity))
                },
            }
        }

        // Apply action outcomes.
        for i in 0..space_count {
            match &outcomes[i] {
                None => {}
                Some(outcome) => self.apply_action_outcome(outcome, rng),
            }
        }
    }

    fn apply_action_outcome(&mut self, outcome: &Outcome, rng: &mut RngBuffer) {
        match outcome {
            Outcome::Wait { .. } => {}
            Outcome::Move { from, direction } => {
                self.resolve_move(*from, *direction, rng);
            }
            Outcome::Turn { location, facing } => {
                self.resolve_turn(*location, *facing, rng);
            }
            Outcome::Blocked => {}
        }
    }

    fn resolve_move(&mut self, location: Location, direction: Direction, rng: &mut RngBuffer) {
        let mut entity = self.remove_entity(location).expect("entity should be at this location");
        entity.set_location(location.plus(direction, &self));
        self.place_entity(entity).expect("this location should be unoccupied");
    }

    fn resolve_turn(&mut self, location: Location, facing: Direction, rng: &mut RngBuffer) {
        let mut entity = self.remove_entity(location).expect("entity should be at this location");
        entity.facing = facing;
        self.place_entity(entity).expect("this location should be unoccupied");
    }

    fn is_occupied(&self, location: Location) -> bool {
        self.get_entity(location).is_some()
    }

    fn get_entity(&self, location: Location) -> Option<&Entity> {
        self.entity_grid[self.buffer_index(location)].as_ref()
    }

    fn remove_entity(&mut self, location: Location) -> Option<Entity> {
        let index = self.buffer_index(location);
        let entity = std::mem::replace(&mut self.entity_grid[index], None);
        return entity;
    }

    fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        if self.is_occupied(entity.location) {
            return Err(());
        }
        let insert_index = self.buffer_index(entity.location);
        self.entity_grid[insert_index] = Some(entity);
        return Ok(());
    }

    fn buffer_index(&self, location: Location) -> usize {
        location.x() + self.width * location.y()
    }
}


struct Conflict {
    north: bool,
    northeast: bool,
    east: bool,
    southeast: bool,
    south: bool,
    southwest: bool,
    west: bool,
    northwest: bool,
}

impl Conflict {
    fn none() -> Conflict {
        Conflict {
            north: false,
            northeast: false,
            east: false,
            southeast: false,
            south: false,
            southwest: false,
            west: false,
            northwest: false,
        }
    }

    fn from_direction(direction: Direction) -> Conflict {
        let mut conflict = Conflict::none();
        match direction {
            Direction::North => conflict.north = true,
            Direction::Northeast => conflict.northeast = true,
            Direction::East => conflict.east = true,
            Direction::Southeast => conflict.southeast = true,
            Direction::South => conflict.south = true,
            Direction::Southwest => conflict.southwest = true,
            Direction::West => conflict.west = true,
            Direction::Northwest => conflict.northwest = true,
        }
        return conflict;
    }

    fn directions(&self) -> Vec<Direction> {
        let mut directions = Vec::with_capacity(8);
        if self.north { directions.push(Direction::North) }
        if self.northeast { directions.push(Direction::Northeast) }
        if self.east { directions.push(Direction::East) }
        if self.southeast { directions.push(Direction::Southeast) }
        if self.south { directions.push(Direction::South) }
        if self.southwest { directions.push(Direction::Southwest) }
        if self.west { directions.push(Direction::West) }
        if self.northwest { directions.push(Direction::Northwest) }
        return directions;
    }

    fn is_conflicted(&self) -> bool {
        let mut count = 0;
        count += self.north as usize;
        count += self.northeast as usize;
        count += self.east as usize;
        count += self.southeast as usize;
        count += self.south as usize;
        count += self.southwest as usize;
        count += self.west as usize;
        count += self.northwest as usize;
        return count > 1;
    }
}

fn init_vec_with<T, F>(mut f: F, capacity: usize) -> Vec<T>
    where F: FnMut() -> T {
    let mut vec: Vec<T> = Vec::with_capacity(capacity);
    vec.resize_with(capacity, f);
    return vec;
}
