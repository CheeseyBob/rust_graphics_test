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

        // Determine entity actions.
        self.actions.fill_with(|| None);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = entity.determine_action(&self.world, rng);
                self.actions.replace(entity.location.coordinates(), Some(action));
            }
        }}

        // Find conflicting actions.
        self.conflicts.fill_with(Conflict::none);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let location = entity.location;
                let action = self.actions.get(location.coordinates()).as_ref()
                    .expect("there should be an action at this location");

                for direction in action.conflicting_directions() {
                    let conflict_location = location.plus(direction, &self.world.entity_grid);
                    self.conflicts.get_mut(conflict_location.coordinates()).add_from(direction);
                }
            }
        }}

        // Resolve conflicts.
        self.outcomes.fill_with(|| None);
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = self.actions.get(entity.location.coordinates()).as_ref()
                    .expect("there should be an action at this location");

                for direction in action.conflicting_directions() {
                    let conflict_direction = entity.location.plus(direction, &self.world.entity_grid);
                    if self.conflicts.get(conflict_direction.coordinates()).is_conflicted() {
                        self.outcomes.replace(entity.location.coordinates(), Some(Outcome::Blocked));
                        break;
                    }
                }
            }
        }}

        // Resolve actions.
        for space in self.world.entity_grid.iter() { match space {
            None => {}
            Some(entity) => {
                let action = self.actions.get(entity.location.coordinates()).as_ref()
                    .expect("there should be an action at this location");
                if self.outcomes.get(entity.location.coordinates()).is_none() {
                    let outcome = action.resolve(&self.world, entity);
                    self.outcomes.replace(entity.location.coordinates(), Some(outcome));
                }
            }
        }}

        // Apply action outcomes.
        for space in self.outcomes.iter() { match space {
            None => {}
            Some(outcome) => {
                apply_action_outcome(&mut self.world, outcome);
            }
        }}
    }
}

fn apply_action_outcome(world: &mut World, outcome: &Outcome) {
    match outcome {
        Outcome::Wait { .. } => {}
        Outcome::Move { from, direction } => {
            resolve_move(world, *from, *direction);
        }
        Outcome::Turn { location, facing } => {
            resolve_turn(world, *location, *facing);
        }
        Outcome::Blocked => {}
    }
}

fn resolve_move(world: &mut World, location: Location, direction: Direction) {
    let mut entity = world.remove_entity(location).expect("entity should be at this location");
    entity.location = location.plus(direction, &world.entity_grid);
    world.place_entity(entity).expect("this location should be unoccupied");
}

fn resolve_turn(world: &mut World, location: Location, facing: Direction) {
    let mut entity = world.remove_entity(location).expect("entity should be at this location");
    entity.facing = facing;
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

    fn add_from(&mut self, direction: Direction) {
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
