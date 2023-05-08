use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;
use crate::world::location::{Direction, Location};


mod location {
    use std::ops::Add;
    use crate::rng_buffer::RngBuffer;
    use crate::world::{Grid, World};

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

        pub fn plus<T>(&self, direction: Direction, grid: &Grid<T>) -> Location {
            Location {
                x: grid.width.wrapping_add_signed(direction.x() as isize).add(self.x) % grid.width,
                y: grid.height.wrapping_add_signed(direction.y() as isize).add(self.y) % grid.height,
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
                let target_location = entity.location.plus(*direction, &world.entity_grid);
                match get_entity(&world.entity_grid, target_location) {
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



pub struct Grid<T> {
    width: usize,
    height: usize,
    grid: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new_filled_using<F>(mut f: F, width: usize, height: usize) -> Grid<T>
        where F: FnMut() -> T {
        Grid {
            grid: init_vec_with(f, width * height),
            width,
            height,
        }
    }

    pub fn get(&self, coordinates: (usize, usize)) -> &T {
        &self.grid[self.index(coordinates)]
    }

    pub fn replace(&mut self, coordinates: (usize, usize), value: T) -> T {
        let index = self.index(coordinates);
        std::mem::replace(&mut self.grid[index], value)
    }

    fn index(&self, coordinates: (usize, usize)) -> usize {
        coordinates.0 + self.width * coordinates.1
    }
}

fn init_vec_with<T, F>(mut f: F, capacity: usize) -> Vec<T>
    where F: FnMut() -> T {
    let mut vec: Vec<T> = Vec::with_capacity(capacity);
    vec.resize_with(capacity, f);
    return vec;
}



pub struct World {
    width: usize,
    height: usize,
    entity_grid: Grid<Option<Entity>>,
    actions: Grid<Option<Action>>,
    conflicts: Grid<Conflict>,
    outcomes: Grid<Option<Outcome>>,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        World {
            entity_grid: Grid::new_filled_using(|| None, width, height),
            actions: Grid::new_filled_using(|| None, width, height),
            conflicts: Grid::new_filled_using(Conflict::none, width,  height),
            outcomes: Grid::new_filled_using(|| None, width, height),
            width,
            height,
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for space in &self.entity_grid.grid {
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
            if place_entity(&mut self.entity_grid, entity).is_ok() {
                count += 1;
            }
        }
    }

    pub fn step(&mut self, rng: &mut RngBuffer) {
        let space_count = self.width * self.height;

        // Determine entity actions.
        self.actions.grid.fill_with(|| None);
        for i in 0..space_count {
            self.actions.grid[i] = match &self.entity_grid.grid[i] {
                Some(entity) => Some(entity.determine_action(&self, rng)),
                None => None,
            };
        }

        // Find conflicting actions.
        self.conflicts.grid.fill_with(Conflict::none);
        for x in 0..self.width {
            for y in 0..self.height {
                match &self.actions.grid[x + y * self.width] {
                    None => {}
                    Some(action) => {
                        let conflict = action.conflicts();
                        let location = Location::new(x, y, &self);

                        let north = location.plus(Direction::North, &self.entity_grid);
                        let northeast = location.plus(Direction::Northeast, &self.entity_grid);
                        let east = location.plus(Direction::East, &self.entity_grid);
                        let southeast = location.plus(Direction::Southeast, &self.entity_grid);
                        let south = location.plus(Direction::South, &self.entity_grid);
                        let southwest = location.plus(Direction::Southwest, &self.entity_grid);
                        let west = location.plus(Direction::West, &self.entity_grid);
                        let northwest = location.plus(Direction::Northwest, &self.entity_grid);

                        let north = self.buffer_index(north);
                        let northeast = self.buffer_index(northeast);
                        let east = self.buffer_index(east);
                        let southeast = self.buffer_index(southeast);
                        let south = self.buffer_index(south);
                        let southwest = self.buffer_index(southwest);
                        let west = self.buffer_index(west);
                        let northwest = self.buffer_index(northwest);

                        self.conflicts.grid[north].south = conflict.north;
                        self.conflicts.grid[northeast].southwest = conflict.northeast;
                        self.conflicts.grid[east].west = conflict.east;
                        self.conflicts.grid[southeast].northwest = conflict.southeast;
                        self.conflicts.grid[south].north = conflict.south;
                        self.conflicts.grid[southwest].northeast = conflict.southwest;
                        self.conflicts.grid[west].east = conflict.west;
                        self.conflicts.grid[northwest].southeast = conflict.northwest;

                    }
                }
            }
        }

        // Resolve conflicts.
        self.outcomes.grid.fill_with(|| None);
        for x in 0..self.width {
            for y in 0..self.height {
                match &self.actions.grid[x + y * self.width] {
                    None => {}
                    Some(action) => {
                        let location = Location::new(x, y, &self);
                        for direction in action.conflicts().directions() {
                            let conflict_direction = location.plus(direction, &self.entity_grid);
                            let index = self.buffer_index(conflict_direction);
                            if self.conflicts.grid[index].is_conflicted() {
                                self.outcomes.grid[x + y * self.width] = Some(Outcome::Blocked);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // Resolve actions.
        for i in 0..space_count {
            if self.outcomes.grid[i].is_some() {
                continue;
            }
            self.outcomes.grid[i] = match &self.actions.grid[i] {
                None => None,
                Some(action) => {
                    let entity = self.entity_grid.grid[i].as_ref().expect("entity should be at this location");
                    Some(action.resolve(&self, entity))
                },
            }
        }

        // Apply action outcomes.
        for i in 0..space_count {
            match &self.outcomes.grid[i] {
                None => {}
                Some(outcome) => apply_action_outcome(&mut self.entity_grid, outcome),
            }
        }
    }

    fn buffer_index(&self, location: Location) -> usize {
        location.x() + self.width * location.y()
    }
}

fn apply_action_outcome(entity_grid: &mut Grid<Option<Entity>>, outcome: &Outcome) {
    match outcome {
        Outcome::Wait { .. } => {}
        Outcome::Move { from, direction } => {
            resolve_move(entity_grid, *from, *direction);
        }
        Outcome::Turn { location, facing } => {
            resolve_turn(entity_grid, *location, *facing);
        }
        Outcome::Blocked => {}
    }
}

fn resolve_move(entity_grid: &mut Grid<Option<Entity>>, location: Location, direction: Direction) {
    let mut entity = remove_entity(entity_grid, location).expect("entity should be at this location");
    entity.set_location(location.plus(direction, entity_grid));
    place_entity(entity_grid, entity).expect("this location should be unoccupied");
}

fn resolve_turn(entity_grid: &mut Grid<Option<Entity>>, location: Location, facing: Direction) {
    let mut entity = remove_entity(entity_grid, location).expect("entity should be at this location");
    entity.facing = facing;
    place_entity(entity_grid, entity).expect("this location should be unoccupied");
}

fn get_entity(entity_grid: &Grid<Option<Entity>>, location: Location) -> Option<&Entity> {
    entity_grid.get(location.coordinates()).as_ref()
}

fn place_entity(entity_grid: &mut Grid<Option<Entity>>, entity: Entity) -> Result<(), ()> {
    match get_entity(entity_grid, entity.location) {
        Some(_) => Err(()),
        None => {
            match entity_grid.replace(entity.location.coordinates(), Some(entity)) {
                None => Ok(()),
                Some(_) => panic!("this space should be unoccupied")
            }
        }
    }
}

fn remove_entity(entity_grid: &mut Grid<Option<Entity>>, location: Location) -> Option<Entity> {
    entity_grid.replace(location.coordinates(), None)
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