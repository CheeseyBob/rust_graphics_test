use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use rayon::prelude::*;
use crate::graphics_window;
use crate::graphics_window::Color;
use crate::grid::{Direction, Grid, Location};
use crate::thread_pool::ThreadPool;
use crate::world::{World};
use crate::world::action::{Action, Outcome};

const PARALLELISM: usize = 10;

static mut WORLD: Option<World> = None;
static mut LOCATIONS: Vec<Location> = Vec::new();
static mut ACTION_GRID: Vec<Option<Action>> = Vec::new();
static mut CONFLICT_GRID: Vec<Conflict> = Vec::new();
static mut OUTCOME_GRID: Vec<Option<Outcome>> = Vec::new();

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

pub fn init(world: World) {
    let size = world.width() * world.height();
    unsafe {
        WORLD = Some(world);
        LOCATIONS = Vec::with_capacity(size);
        ACTION_GRID = Vec::with_capacity(size);
        ACTION_GRID.fill_with(|| { None });
        CONFLICT_GRID = Vec::with_capacity(size);
        CONFLICT_GRID.fill_with(|| { Conflict::none() });
        OUTCOME_GRID = Vec::with_capacity(size);
        OUTCOME_GRID.fill_with(|| { None });
    }
}

pub fn draw() {
    graphics_window::clear(Color::BLACK);

    unsafe { WORLD.as_ref() }.unwrap().iter_entities_par().for_each(|entity| {
        let location = &entity.location;
        let x = location.x();
        let y = location.y();
        let pixel_color = entity.pixel_color();
        graphics_window::draw_pixel(x, y, pixel_color);
    });
}

pub fn step() {

    get_locations_for_processing();

    determine_actions();

    resolve_conflicts();

    determine_outcomes();

    apply_outcomes();

    clean_up();
}

fn get_locations_for_processing() {
    unsafe {
        LOCATIONS = WORLD.as_ref().unwrap().iter_entities_par()
            .map(|entity| entity.location)
            .collect();
    }
}

fn determine_actions() {
    unsafe { &LOCATIONS }.par_iter().for_each(|location| {
        determine_action_for_location(location);
    });
}

fn resolve_conflicts() {
    unsafe { &LOCATIONS }.par_iter().for_each(|location| {
        resolve_conflicts_for_location(location);
    });
}

fn determine_outcomes() {
    unsafe { &LOCATIONS }.par_iter().for_each(|location| {
        determine_outcomes_for_location(location);
    });
}

fn apply_outcomes() {
    unsafe { &LOCATIONS }.par_iter().for_each(|location| {
        apply_outcome_for_location(location);
    });
}

fn clean_up() {
    unsafe { &LOCATIONS }.par_iter().for_each(|location| {
        unsafe {
            action_at_mut(location).take();
            conflict_at_mut(location).clear(); // Why does replacing this with the proper clean-up seem to break things?
            outcome_at_mut(location).take();
        }
    });

    unsafe {
        LOCATIONS.clear();
    }
}

fn determine_actions_for_slice(locations: &[Location]) {
    for location in locations {
        determine_action_for_location(location);
    }
}

fn determine_action_for_location(location: &Location) {
    let entity = unsafe {
        WORLD.as_ref().unwrap().get_entity(location)
            .expect("entity should be at this location")
    };
    let action = entity.determine_action(unsafe { WORLD.as_ref().unwrap() });

    match action.conflicting_directions() {
        None => {}
        Some(directions) => for direction in directions {
            let conflict_location = unsafe {
                WORLD.as_ref().unwrap().add(&entity.location, &direction)
            };
            unsafe { conflict_at_mut(&conflict_location) }.add_from(&direction);
        }
    }

    unsafe {
        action_at_mut(&entity.location).replace(action);
    }
}

fn resolve_conflicts_for_slice(locations: &[Location]) {
    for location in locations {
        resolve_conflicts_for_location(location);
    }
}

fn resolve_conflicts_for_location(location: &Location) {
    let action = unsafe { action_at(location) }.as_ref()
        .expect("there should be an action at this location");
    match action.conflicting_directions() {
        None => {}
        Some(directions) => for direction in &directions {
            let conflict_direction = unsafe { WORLD.as_ref() }
                .unwrap().add(location, direction);
            if unsafe { conflict_at(&conflict_direction) }.is_conflicted() {
                unsafe { outcome_at_mut(location) }.replace(Outcome::Blocked);
                break;
            }
        }
    }
}

fn determine_outcomes_for_slice(locations: &[Location]) {
    for location in locations {
        determine_outcomes_for_location(location);
    }
}

fn determine_outcomes_for_location(location: &Location) {
    let entity = unsafe {
        WORLD.as_ref().unwrap().get_entity(location)
            .expect("entity should be at this location")
    };
    let action = unsafe { action_at(location) }.as_ref()
        .expect("there should be an action at this location");
    if unsafe { outcome_at(location) }.is_none() { // Otherwise the outcome here is from conflict resolution, which takes precedence.
        let outcome = action.resolve(entity, unsafe { WORLD.as_ref().unwrap() });
        unsafe { outcome_at_mut(location) }.replace(outcome);
    }
}

fn apply_outcomes_for_slice(locations: &[Location]) {
    for location in locations {
        apply_outcome_for_location(location);
    }
}

fn apply_outcome_for_location(location: &Location) {
    let outcome = unsafe { outcome_at(location) }.as_ref()
        .expect("there should be an outcome at this location");

    match outcome {
        Outcome::Wait => {}
        Outcome::Move(direction) => resolve_move(location, direction),
        Outcome::Turn(facing) => resolve_turn(location, facing),
        Outcome::Blocked => {}
    }
}

fn resolve_move(location: &Location, direction: &Direction) {
    unsafe {
        WORLD.as_mut().unwrap().move_entity(location, direction)
            .expect("entity should be at location and destination should be unoccupied");
    }
}

fn resolve_turn(location: &Location, facing: &Direction) {
    let mut entity = unsafe {
        WORLD.as_mut().unwrap().get_entity_mut(location)
            .expect("entity should be at this location")
    };
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
        self.south = false;
        self.southwest = false;
        self.west = false;
        self.northwest = false;
        self.north = false;
        self.northeast = false;
        self.east = false;
        self.southeast = false;
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
