use crate::grid::{Direction, Grid, Location};
use crate::rng_buffer::RngBuffer;
use crate::world::{Action, Entity, Outcome, World};

pub struct WorldProcessor {
    world: World,
    actions: Grid<Option<Action>>,
    conflicts: Grid<Conflict>,
    outcomes: Grid<Option<Outcome>>,
}

impl WorldProcessor {
    getter_ref!(world: World);

    pub fn new(world: World) -> WorldProcessor {
        WorldProcessor {
            actions: Grid::new_filled_with(|| None, world.width(), world.height()),
            conflicts: Grid::new_filled_with(Conflict::none, world.width(), world.height()),
            outcomes: Grid::new_filled_with(|| None, world.width(), world.height()),
            world,
        }
    }

    pub fn step(&mut self, rng: &mut RngBuffer) {

        // This is temporary - World should be updated to track entities in both a Vec and Grid simultaneously.
        let locations_to_process: Vec<Location> = self.world.entity_grid.iter()
            .filter_map(f!{opt -> opt.as_ref()})
            .map(f!{entity -> entity.location})
            .collect();

        // Determine entity actions.
        self.actions.fill_with(|| None);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = entity.determine_action(&self.world, rng);
                self.actions.replace(&entity.location.clone(), Some(action));
            }
        }}

        // Find conflicting actions.
        self.conflicts.fill_with(Conflict::none);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let location = entity.location;
                let action = self.actions.get(&location).as_ref()
                    .expect("there should be an action at this location");

                match action.conflicting_directions() {
                    None => {}
                    Some(directions) => for direction in directions {
                        let conflict_location = self.world.entity_grid.add(&location, &direction);
                        self.conflicts.get_mut(&conflict_location).add_from(&direction);
                    }
                }
            }
        }}

        // Resolve conflicts.
        self.outcomes.fill_with(|| None);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = self.actions.get(&entity.location).as_ref()
                    .expect("there should be an action at this location");

                match action.conflicting_directions() {
                    None => {}
                    Some(directions) => for direction in directions {
                        let conflict_direction = self.world.entity_grid.add(&entity.location, &direction);
                        if self.conflicts.get(&conflict_direction).is_conflicted() {
                            self.outcomes.replace(&entity.location, Some(Outcome::Blocked));
                            break;
                        }
                    }
                }
            }
        }}

        // Resolve actions.
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = self.actions.get(&entity.location).as_ref()
                    .expect("there should be an action at this location");
                if self.outcomes.get(&entity.location).is_none() {
                    let outcome = action.resolve(entity, &self.world);
                    self.outcomes.replace(&entity.location, Some(outcome));
                }
            }
        }}

        // Apply action outcomes.
        for location in locations_to_process {
            let outcome = self.outcomes.get(&location).as_ref()
                .expect("there should be an outcome at this location");
            apply_action_outcome(outcome, &location, &mut self.world);
        }
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
    let mut entity = world.remove_entity(location).expect("entity should be at this location");
    entity.location = world.entity_grid.add(&location, &direction);
    world.place_entity(entity).expect("this location should be unoccupied");
}

fn resolve_turn(location: &Location, facing: &Direction, world: &mut World) {
    let mut entity = world.remove_entity(location).expect("entity should be at this location");
    entity.facing = *facing;
    world.place_entity(entity).expect("this location should be unoccupied");
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
