use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::graphics_window::GraphicsBuffer;
use crate::grid::{Direction, Grid, Location};
use crate::thread_pool::ThreadPool;
use crate::world::{World};
use crate::world::action::{Action, Outcome};

static mut WORLD: Option<World> = None;

pub struct WorldProcessor {
    locations: Vec<Location>,
    actions: ActionGrid,
    conflicts: ConflictGrid,
    outcomes: OutcomeGrid,
}

type ActionGrid = Arc<Grid<Option<Action>>>;
type ConflictGrid = Arc<Grid<Conflict>>;
type OutcomeGrid = Arc<Grid<Option<Outcome>>>;
const PARALLELISM: usize = 10;

pub fn init(world: World) -> WorldProcessor {
    let (width, height) = (world.width(), world.height());
    unsafe {
        WORLD = Some(world);
    }
    WorldProcessor {
        locations: Vec::with_capacity(width * height),
        actions: Arc::new(Grid::new_filled_with(|| None, width, height)),
        conflicts: Arc::new(Grid::new_filled_with(Conflict::none, width, height)),
        outcomes: Arc::new(Grid::new_filled_with(|| None, width, height)),
    }
}

pub fn draw(graphics_buffer: &mut GraphicsBuffer) {
    unsafe {
        if let Some(world) = &WORLD {
            world.draw(graphics_buffer);
        }
    }
}

impl WorldProcessor {
    pub fn step(&mut self) {

        self.get_locations_for_processing();

        self.determine_actions();

        self.resolve_conflicts();

        self.determine_outcomes();

        self.apply_outcomes();

        self.clean_up();
    }

    fn get_locations_for_processing(&mut self) {
        unsafe {
            for entity in WORLD.as_ref().unwrap().iter_entities() {
                self.locations.push(entity.location);
            }
        }
    }

    fn determine_actions(&self) {
        thread::scope(|scope| {
            let parallelism = PARALLELISM;
            let total_length = self.locations.len();
            let slice_length = total_length / parallelism;
            for i in 0..parallelism {
                let slice_start = i * slice_length;
                let slice_end = total_length - (parallelism - i - 1) * slice_length;
                let locations = &self.locations[slice_start..slice_end];
                let actions = self.actions.clone();
                let conflicts = self.conflicts.clone();
                scope.spawn(move || {
                    determine_actions_for_slice(locations, actions, conflicts);
                });
            }
        });
    }

    fn resolve_conflicts(&self) {
        thread::scope(|scope| {
            let parallelism = PARALLELISM;
            let total_length = self.locations.len();
            let slice_length = total_length / parallelism;
            for i in 0..parallelism {
                let slice_start = i * slice_length;
                let slice_end = total_length - (parallelism - i - 1) * slice_length;
                let locations = &self.locations[slice_start..slice_end];
                let actions = self.actions.clone();
                let conflicts = self.conflicts.clone();
                let outcomes = self.outcomes.clone();
                scope.spawn(move || {
                    resolve_conflicts_for_slice(locations, actions, conflicts, outcomes);
                });
            }
        });
    }

    fn determine_outcomes(&self) {
        thread::scope(|scope| {
            let parallelism = PARALLELISM;
            let total_length = self.locations.len();
            let slice_length = total_length / parallelism;
            for i in 0..parallelism {
                let slice_start = i * slice_length;
                let slice_end = total_length - (parallelism - i - 1) * slice_length;
                let locations = &self.locations[slice_start..slice_end];
                let actions = self.actions.clone();
                let outcomes = self.outcomes.clone();
                scope.spawn(move || {
                    determine_outcomes_for_slice(locations, actions, outcomes);
                });
            }
        });
    }

    fn apply_outcomes(&self) {
        thread::scope(|scope| {
            let parallelism = PARALLELISM;
            let total_length = self.locations.len();
            let slice_length = total_length / parallelism;
            for i in 0..parallelism {
                let slice_start = i * slice_length;
                let slice_end = total_length - (parallelism - i - 1) * slice_length;
                let locations = &self.locations[slice_start..slice_end];
                let outcomes = self.outcomes.clone();
                scope.spawn(move || {
                    apply_outcomes_for_slice(locations, outcomes);
                });
            }
        });
    }

    fn clean_up(&mut self) {

        /*
        thread::scope(|scope| {

            scope.spawn(|| {
                for location in &self.locations {
                    self.actions.replace(location, None);
                }
            });
            scope.spawn(|| {
                for location in &self.locations {
                    self.conflicts.replace(location, Conflict::none());
                }
            });
            scope.spawn(|| {
                for location in &self.locations {
                    self.outcomes.replace(location, None);
                }
            });

        });
        */

        for location in &self.locations {
            self.actions.get(location).take();
            self.conflicts.get(location).clear();
            self.outcomes.get(location).take();
        }
        self.locations.clear();
    }
}

fn determine_actions_for_slice(locations: &[Location], actions: ActionGrid, conflicts: ConflictGrid) {
    for location in locations {
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
                conflicts.get(&conflict_location).add_from(&direction);
            }
        }

        actions.get(&entity.location).replace(action);
    }
}

fn resolve_conflicts_for_slice(locations: &[Location], actions: ActionGrid, conflicts: ConflictGrid, outcomes: OutcomeGrid) {
    for location in locations {
        let guard = actions.get(&location);
        let action = guard.as_ref()
            .expect("there should be an action at this location");
        match action.conflicting_directions() {
            None => {}
            Some(directions) => for direction in &directions {
                let conflict_direction = unsafe {
                    WORLD.as_ref().unwrap().add(location, direction)
                };
                if conflicts.get(&conflict_direction).is_conflicted() {
                    outcomes.get(location).replace(Outcome::Blocked);
                    break;
                }
            }
        }
    }
}

fn determine_outcomes_for_slice(locations: &[Location], actions: ActionGrid, outcomes: OutcomeGrid) {
    for location in locations {
        let entity = unsafe {
            WORLD.as_ref().unwrap().get_entity(location)
                .expect("entity should be at this location")
        };
        let guard = actions.get(location);
        let action = guard.as_ref()
            .expect("there should be an action at this location");

        if outcomes.get(location).is_none() { // Otherwise the outcome here is from conflict resolution, which takes precedence.
            let outcome = action.resolve(entity, unsafe { WORLD.as_ref().unwrap() });
            outcomes.get(location).replace(outcome);
        }
    }
}

fn apply_outcomes_for_slice(locations: &[Location], outcomes: OutcomeGrid) {
    for location in locations {
        let guard = outcomes.get(location);
        let outcome = guard.as_ref()
            .expect("there should be an outcome at this location");

        apply_outcome_for_location(location, outcome);
    }
}

fn apply_outcome_for_location(location: &Location, outcome: &Outcome) {
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
