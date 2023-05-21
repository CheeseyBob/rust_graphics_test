use rayon::prelude::*;
use crate::graphics_window;
use crate::graphics_window::Color;
use crate::world::{Direction, Location, World};
use crate::action::{Action, Outcome};

static mut WORLD: Option<World> = None;
static mut LOCATIONS: Vec<Location> = Vec::new();
static mut ACTION_GRID: Vec<Option<Action>> = Vec::new();
static mut CONFLICT_GRID: Vec<Conflict> = Vec::new();
static mut OUTCOME_GRID: Vec<Option<Outcome>> = Vec::new();
static mut DRAWING_ENABLED: bool = true;

fn is_drawing_enabled() -> bool {
    unsafe { DRAWING_ENABLED }
}

fn is_initialised() -> bool {
    unsafe { WORLD.is_some() }
}

unsafe fn action_at(location: &Location) -> &Option<Action> {
    ACTION_GRID.get_unchecked(location.index())
}

unsafe fn action_at_mut(location: &Location) -> &mut Option<Action> {
    ACTION_GRID.get_unchecked_mut(location.index())
}

unsafe fn conflict_at(location: &Location) -> &Conflict {
    CONFLICT_GRID.get_unchecked(location.index())
}

unsafe fn conflict_at_mut(location: &Location) -> &mut Conflict {
    CONFLICT_GRID.get_unchecked_mut(location.index())
}

unsafe fn outcome_at(location: &Location) -> &Option<Outcome> {
    OUTCOME_GRID.get_unchecked(location.index())
}

unsafe fn outcome_at_mut(location: &Location) -> &mut Option<Outcome> {
    OUTCOME_GRID.get_unchecked_mut(location.index())
}

pub fn init(world: World) -> Result<(), ()> {
    if is_initialised() { return Err(()) }

    let size = world.width() * world.height();
    unsafe {
        WORLD = Some(world);
        LOCATIONS = Vec::with_capacity(size);
        ACTION_GRID = Vec::with_capacity(size);
        ACTION_GRID.resize_with(size, || { None });
        CONFLICT_GRID = Vec::with_capacity(size);
        CONFLICT_GRID.resize_with(size, || { Conflict::none() });
        OUTCOME_GRID = Vec::with_capacity(size);
        OUTCOME_GRID.resize_with(size, || { None });
    }
    Ok(())
}

/// Safety: Don't call while WORLD is being mutated.
pub unsafe fn draw() {
    if !is_drawing_enabled() { return }

    graphics_window::clear(Color::BLACK);

    WORLD.as_ref().unwrap().iter_entities_par().for_each(|entity| {
        let location = &entity.location;
        let x = location.x();
        let y = location.y();
        let pixel_color = entity.pixel_color();
        graphics_window::draw_pixel(x, y, pixel_color);
    });
}

pub fn step() {
    unsafe {
        let draw_thread = std::thread::spawn(|| {
            // Safety:
            // Reads: WORLD
            // Mutates: -
            draw();
        });

        // Safety:
        // Reads: -
        // Mutates: LOCATIONS, ACTION_GRID, CONFLICT_GRID, OUTCOME_GRID
        clean_up();

        // Safety:
        // Reads: WORLD
        // Mutates: LOCATIONS
        get_locations_for_processing();

        // Safety:
        // Reads: LOCATIONS, WORLD
        // Mutates: ACTION_GRID, CONFLICT_GRID
        determine_actions();

        // Safety:
        // Reads: LOCATIONS, ACTION_GRID, CONFLICT_GRID
        // Mutates: OUTCOME_GRID
        resolve_conflicts();

        // Safety:
        // Reads: LOCATIONS, WORLD, ACTION_GRID
        // Mutates: OUTCOME_GRID
        determine_outcomes();

        // Safety: WORLD will no longer be read by draw thread.
        draw_thread.join().unwrap();

        // Safety:
        // Reads: LOCATIONS, OUTCOME_GRID
        // Mutates: WORLD
        apply_outcomes()
    }
}

/// Safety: This function mutates LOCATIONS, ACTION_GRID, CONFLICT_GRID and OUTCOME_GRID.
unsafe fn clean_up() {
    LOCATIONS.par_iter().for_each(|location| {
        action_at_mut(location).take();
        outcome_at_mut(location).take();
    });
    CONFLICT_GRID.par_iter_mut().for_each(|conflict| {
        conflict.clear()
    });
    LOCATIONS.clear();
}

/// Safety: This function reads from WORLD and mutates LOCATIONS;
unsafe fn get_locations_for_processing() {
    LOCATIONS = WORLD.as_ref().unwrap().iter_entities_par()
        .map(|entity| entity.location)
        .collect();
}

/// Safety: This function reads from LOCATIONS and WORLD and mutates ACTION_GRID and CONFLICT_GRID.
unsafe fn determine_actions() {
    LOCATIONS.par_iter().for_each(|location| {
        determine_action_for_location(location);
    });
}

/// Safety: This function reads from LOCATIONS, ACTION_GRID and CONFLICT_GRID and mutates OUTCOME_GRID.
unsafe fn resolve_conflicts() {
    LOCATIONS.par_iter().for_each(|location| {
        resolve_conflicts_for_location(location);
    });
}

/// Safety: This function reads from LOCATIONS, WORLD and ACTION_GRID and mutates OUTCOME_GRID.
unsafe fn determine_outcomes() {
    LOCATIONS.par_iter().for_each(|location| {
        determine_outcomes_for_location(location);
    });
}

/// Safety: This function reads from LOCATIONS and OUTCOME_GRID and mutates WORLD.
unsafe fn apply_outcomes() {
    LOCATIONS.par_iter().for_each(|location| {
        apply_outcome_for_location(location);
    });
}

/// Safety: This function reads from WORLD and mutates ACTION_GRID and CONFLICT_GRID.
unsafe fn determine_action_for_location(location: &Location) {
    let entity =WORLD.as_ref().unwrap().get_entity(location)
            .expect("entity should be at this location");
    let action = entity.determine_action(WORLD.as_ref().unwrap());

    match action.conflicting_directions() {
        None => {}
        Some(directions) => for direction in directions {
            let conflict_location = WORLD.as_ref().unwrap().add(&entity.location, &direction);
           conflict_at_mut(&conflict_location).add_from(&direction);
        }
    }

    action_at_mut(&entity.location).replace(action);
}

/// Safety: This function reads from ACTION_GRID and CONFLICT_GRID and mutates OUTCOME_GRID.
unsafe fn resolve_conflicts_for_location(location: &Location) {
    let action = action_at(location).as_ref()
        .expect("there should be an action at this location");
    match action.conflicting_directions() {
        None => {}
        Some(directions) => for direction in &directions {
            let conflict_direction = WORLD.as_ref().unwrap().add(location, direction);
            if conflict_at(&conflict_direction).is_conflicted() {
                outcome_at_mut(location).replace(Outcome::Blocked);
                break;
            }
        }
    }
}

/// Safety: This function reads from WORLD and ACTION_GRID and mutates OUTCOME_GRID.
unsafe fn determine_outcomes_for_location(location: &Location) {
    let entity = WORLD.as_ref().unwrap().get_entity(location)
            .expect("entity should be at this location");
    let action = action_at(location).as_ref()
        .expect("there should be an action at this location");
    if outcome_at(location).is_none() { // Otherwise the outcome here is from conflict resolution, which takes precedence.
        let outcome = action.resolve(entity, WORLD.as_ref().unwrap());
        outcome_at_mut(location).replace(outcome);
    }
}

/// Safety: This function reads from OUTCOME_GRID and mutates WORLD.
unsafe fn apply_outcome_for_location(location: &Location) {
    let outcome = outcome_at(location).as_ref()
        .expect("there should be an outcome at this location");

    match outcome {
        Outcome::Wait => {}
        Outcome::Move(direction) => resolve_move(location, direction),
        Outcome::Turn(facing) => resolve_turn(location, facing),
        Outcome::Blocked => {}
    }
}

/// Safety: This function mutates WORLD.
unsafe fn resolve_move(location: &Location, direction: &Direction) {
    WORLD.as_mut().unwrap().move_entity(location, direction)
            .expect("entity should be at location and destination should be unoccupied");
}

/// Safety: This function mutates WORLD.
unsafe fn resolve_turn(location: &Location, facing: &Direction) {
    let mut entity = WORLD.as_mut().unwrap().get_entity_mut(location)
            .expect("entity should be at this location");
    entity.facing = *facing;
}

#[derive(Default)]
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
    #[inline]
    fn none() -> Conflict {
        Conflict::default()
    }

    fn add_from(&mut self, direction: &Direction) {
        match direction {
            Direction::North        => self.south = true,
            Direction::Northeast    => self.southwest = true,
            Direction::East         => self.west = true,
            Direction::Southeast    => self.northwest = true,
            Direction::South        => self.north = true,
            Direction::Southwest    => self.northeast = true,
            Direction::West         => self.east = true,
            Direction::Northwest    => self.southeast = true,
        }
    }

    fn clear(&mut self) {
        self.north = false;
        self.northeast = false;
        self.east = false;
        self.southeast = false;
        self.south = false;
        self.southwest = false;
        self.west = false;
        self.northwest = false;
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
