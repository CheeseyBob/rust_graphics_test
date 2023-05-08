use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;
use crate::grid::{Direction, Grid, Location};

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
            location: Location::new(x, y, &world.entity_grid),
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
            entity_grid: Grid::new_filled_with(|| None, width, height),
            actions: Grid::new_filled_with(|| None, width, height),
            conflicts: Grid::new_filled_with(Conflict::none, width, height),
            outcomes: Grid::new_filled_with(|| None, width, height),
            width,
            height,
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for space in self.entity_grid.iter() {
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
}


pub fn step(world: &mut World, rng: &mut RngBuffer) {

    // Determine entity actions.
    world.actions.fill_with(|| None);
    for space in world.entity_grid.iter() { match space {
        None => {}
        Some(entity) => {
            let action = entity.determine_action(&world, rng);
            world.actions.replace(entity.location.coordinates(), Some(action));
        }
    }}

    // Find conflicting actions.
    world.conflicts.fill_with(Conflict::none);
    for space in world.entity_grid.iter() { match space {
        None => {}
        Some(entity) => {
            let location = entity.location;
            let action = world.actions.get(location.coordinates()).as_ref()
                .expect("there should be an action at this location");
            let conflict = action.conflicts();

            let north = location.plus(Direction::North, &world.entity_grid);
            let northeast = location.plus(Direction::Northeast, &world.entity_grid);
            let east = location.plus(Direction::East, &world.entity_grid);
            let southeast = location.plus(Direction::Southeast, &world.entity_grid);
            let south = location.plus(Direction::South, &world.entity_grid);
            let southwest = location.plus(Direction::Southwest, &world.entity_grid);
            let west = location.plus(Direction::West, &world.entity_grid);
            let northwest = location.plus(Direction::Northwest, &world.entity_grid);

            world.conflicts.get_mut(north.coordinates()).south = conflict.north;
            world.conflicts.get_mut(northeast.coordinates()).southwest = conflict.northeast;
            world.conflicts.get_mut(east.coordinates()).west = conflict.east;
            world.conflicts.get_mut(southeast.coordinates()).northwest = conflict.southeast;
            world.conflicts.get_mut(south.coordinates()).north = conflict.south;
            world.conflicts.get_mut(southwest.coordinates()).northeast = conflict.southwest;
            world.conflicts.get_mut(west.coordinates()).east = conflict.west;
            world.conflicts.get_mut(northwest.coordinates()).southeast = conflict.northwest;
        }
    }}

    // Resolve conflicts.
    world.outcomes.fill_with(|| None);
    for space in world.entity_grid.iter() { match space {
        None => {}
        Some(entity) => {
            let action = world.actions.get(entity.location.coordinates()).as_ref()
                .expect("there should be an action at this location");

            for direction in action.conflicts().directions() {
                let conflict_direction = entity.location.plus(direction, &world.entity_grid);
                if world.conflicts.get(conflict_direction.coordinates()).is_conflicted() {
                    world.outcomes.replace(entity.location.coordinates(), Some(Outcome::Blocked));
                    break;
                }
            }
        }
    }}

    // Resolve actions.
    for space in world.entity_grid.iter() { match space {
        None => {}
        Some(entity) => {
            let action = world.actions.get(entity.location.coordinates()).as_ref()
                .expect("there should be an action at this location");
            if world.outcomes.get(entity.location.coordinates()).is_none() {
                let outcome = action.resolve(&world, entity);
                world.outcomes.replace(entity.location.coordinates(), Some(outcome));
            }
        }
    }}

    // Apply action outcomes.
    for space in world.outcomes.iter() { match space {
        None => {}
        Some(outcome) => {
            apply_action_outcome(&mut world.entity_grid, outcome);
        }
    }}
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