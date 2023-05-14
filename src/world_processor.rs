use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::grid::{Direction, Grid, Location};
use crate::rng_buffer::RngBuffer;
use crate::thread_pool::ThreadPool;
use crate::world::{World};
use crate::world::action::{Action, Outcome};

pub struct WorldProcessor {
    locations: Vec<Location>,
    actions: ActionGrid,
    conflicts: ConflictGrid,
    outcomes: Grid<Option<Outcome>>,
    thread_pool: ThreadPool
}

type ActionGrid = Arc<Grid<Option<Action>>>;
type ConflictGrid = Arc<Grid<Conflict>>;

impl WorldProcessor {
    pub fn new(world: &World) -> WorldProcessor {
        WorldProcessor {
            locations: Vec::with_capacity(world.width() * world.height()),
            actions: Arc::new(Grid::new_filled_with(|| None, world.width(), world.height())),
            conflicts: Arc::new(Grid::new_filled_with(Conflict::none, world.width(), world.height())),
            outcomes: Grid::new_filled_with(|| None, world.width(), world.height()),
            thread_pool: ThreadPool::new(5),
        }
    }

    pub fn step(&mut self, world: &mut World) {

        self.get_locations_for_processing(world);

        self.determine_actions(world);

        self.resolve_conflicts(world);

        self.resolve_actions(world);

        self.apply_outcomes(world);

        self.clean_up();
    }

    fn get_locations_for_processing(&mut self, world: &World) {
        for entity in world.iter_entities() {
            self.locations.push(entity.location);
        }
    }

    fn determine_actions(&self, world: &World) {
        thread::scope(|scope| {
            let parallelism = 10;
            let total_length = self.locations.len();
            let slice_length = total_length / parallelism;
            for i in 0..parallelism {
                let slice_start = i * slice_length;
                let slice_end = total_length - (parallelism - i - 1) * slice_length;
                let locations = &self.locations[slice_start..slice_end];
                let actions = self.actions.clone();
                let conflicts = self.conflicts.clone();
                scope.spawn(move || {
                    determine_actions_for_slice(locations, world, actions, conflicts);
                });
            }
        });
    }

    fn resolve_conflicts(&self, world: &World) {
        thread::scope(|scope| {

            scope.spawn(move || {

            });
        });
        for location in &self.locations {
            let guard = self.actions.get(&location);
            let action = guard.as_ref()
                .expect("there should be an action at this location");
            match action.conflicting_directions() {
                None => {}
                Some(directions) => for direction in &directions {
                    let conflict_direction = world.add(location, direction);
                    if self.conflicts.get(&conflict_direction).is_conflicted() {
                        self.outcomes.get(location).replace(Outcome::Blocked);
                        break;
                    }
                }
            }
        }
    }

    fn resolve_actions(&self, world: &World) {
        for location in &self.locations {
            let entity = world.get_entity(location)
                .expect("entity should be at this location");
            let guard = self.actions.get(location);
            let action = guard.as_ref()
                .expect("there should be an action at this location");

            if self.outcomes.get(location).is_none() {
                let outcome = action.resolve(entity, &world);
                self.outcomes.get(location).replace(outcome);
            }
        }
    }

    fn apply_outcomes(&self, world: &mut World) {
        for location in &self.locations {
            let guard = self.outcomes.get(location);
            let outcome = guard.as_ref()
                .expect("there should be an outcome at this location");

            apply_action_outcome(outcome, location, world);
        }
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

fn determine_actions_for_slice(locations: &[Location], world: &World, actions: ActionGrid, conflicts: ConflictGrid) {
    for location in locations {
        let entity = world.get_entity(location)
            .expect("entity should be at this location");
        let action = entity.determine_action(world);

        match action.conflicting_directions() {
            None => {}
            Some(directions) => for direction in directions {
                let conflict_location = world.add(&entity.location, &direction);
                conflicts.get(&conflict_location).add_from(&direction);
            }
        }

        actions.get(&entity.location).replace(action);
    }
}

fn apply_action_outcome(outcome: &Outcome, location: &Location, world: &mut World) {
    match outcome {
        Outcome::Wait => {}
        Outcome::Move(direction) => resolve_move(location, direction, world),
        Outcome::Turn(facing) => resolve_turn(location, facing, world),
        Outcome::Blocked => {}
    }
}

fn resolve_move(location: &Location, direction: &Direction, world: &mut World) {
    world.move_entity(location, direction)
        .expect("entity should be at location and destination should be unoccupied");
}

fn resolve_turn(location: &Location, facing: &Direction, world: &mut World) {
    let mut entity = world.get_entity_mut(location)
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
