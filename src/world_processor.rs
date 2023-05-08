use crate::grid::{Direction, Grid, Location};
use crate::rng_buffer::RngBuffer;
use crate::world::{World};
use crate::world::action::{Action, Outcome};

pub struct WorldProcessor {
    world: World,
    locations: Vec<Location>,
    actions: Grid<Option<Action>>,
    conflicts: Grid<Conflict>,
    outcomes: Grid<Option<Outcome>>,
}

impl WorldProcessor {
    getter_ref!(world: World);

    pub fn new(world: World) -> WorldProcessor {
        WorldProcessor {
            locations: Vec::with_capacity(world.width() * world.height()),
            actions: Grid::new_filled_with(|| None, world.width(), world.height()),
            conflicts: Grid::new_filled_with(Conflict::none, world.width(), world.height()),
            outcomes: Grid::new_filled_with(|| None, world.width(), world.height()),
            world,
        }
    }

    pub fn step(&mut self, rng: &mut RngBuffer) {

        for entity in self.world.iter_entities() {
            self.locations.push(entity.location);
            let action = entity.determine_action(&self.world, rng);
            self.actions.replace(&entity.location.clone(), Some(action));
        }

        // Find conflicting actions.
        for location in &self.locations {
            let action = self.actions.get(location).as_ref()
                .expect("there should be an action at this location");

            match action.conflicting_directions() {
                None => {}
                Some(directions) => for direction in directions {
                    let conflict_location = self.world.add(&location, &direction);
                    self.conflicts.get_mut(&conflict_location).add_from(&direction);
                }
            }
        }

        // Resolve conflicts.
        for location in &self.locations {
            let action = self.actions.get(location).as_ref()
                .expect("there should be an action at this location");

            match action.conflicting_directions() {
                None => {}
                Some(directions) => for direction in &directions {
                    let conflict_direction = self.world.add(location, direction);
                    if self.conflicts.get(&conflict_direction).is_conflicted() {
                        self.outcomes.replace(location, Some(Outcome::Blocked));
                        break;
                    }
                }
            }
        }

        // Resolve actions.
        for location in &self.locations {
            let entity = self.world.get_entity(location)
                .expect("entity should be at this location");
            let action = self.actions.get(location).as_ref()
                .expect("there should be an action at this location");

            if self.outcomes.get(location).is_none() {
                let outcome = action.resolve(entity, &self.world);
                self.outcomes.replace(location, Some(outcome));
            }
        }

        // Apply action outcomes.
        for location in &self.locations {
            let outcome = self.outcomes.get(location).as_ref()
                .expect("there should be an outcome at this location");
            apply_action_outcome(outcome, location, &mut self.world);
        }

        // Clean up.
        for location in &self.locations {
            self.actions.replace(location, None);
            self.conflicts.replace(location, Conflict::none());
            self.outcomes.replace(location, None);
        }
        self.locations.clear();
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
