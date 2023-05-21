use std::ops::Deref;
use rayon::prelude::*;
use rayon::slice::{Iter, Windows};
use crate::graphics_window::Color;
use crate::grid::{Direction, Grid, Location};
use crate::world::entity::Entity;

pub mod entity {
    use std::sync::Arc;
    use crate::graphics_window::Color;
    use crate::grid::{Direction, Location};
    use crate::rng_buffer;
    use crate::world::action::Action;
    use crate::world::World;

    pub struct Entity {
        pub location: Location,
        pub facing: Direction,
    }

    impl Entity {
        pub fn pixel_color(&self) -> Color {
            Color::WHITE
        }

        pub fn new(x: usize, y: usize, world: &World) -> Entity {
            Entity {
                location: Location::at(x, y, &world.entity_grid),
                facing: Direction::random(),
            }
        }

        pub fn determine_action(&self, world: &World) -> Action {
            match rng_buffer::next() {
                roll if roll < 0.05 => Action::Turn(Direction::random()),
                roll if roll < 0.95 => Action::Move(self.facing),
                _ => Action::Wait,
            }
        }

        pub fn step(&self) {

            // TODO ...

        }
    }
}

pub mod action {
    use crate::grid::Direction;
    use crate::world::entity::Entity;
    use crate::world::World;

    pub enum Action {
        Wait,
        Move(Direction),
        Turn(Direction),
    }

    pub enum Outcome {
        Blocked,
        Wait,
        Move(Direction),
        Turn(Direction),
    }

    impl Action {
        pub fn conflicting_directions(&self) -> Option<Vec<Direction>> {
            match self {
                Action::Wait => None,
                Action::Move(direction) => Some(vec![*direction]),
                Action::Turn(_) => None,
            }
        }

        pub fn resolve(&self, entity: &Entity, world: &World) -> Outcome {
            match self {
                Action::Wait => Outcome::Wait,
                Action::Move(direction) => {
                    let target_location = world.entity_grid.add(&entity.location, direction);
                    match world.get_entity(&target_location) {
                        Some(_) => Outcome::Blocked,
                        None => Outcome::Move(*direction),
                    }
                }
                Action::Turn(facing) => Outcome::Turn(*facing),
            }
        }
    }
}


type EntityId = usize;

pub struct World {
    width: usize,
    height: usize,
    entity_grid: Grid<Option<EntityId>>,
    entities: Vec<Entity>,
}

impl World {
    getter!(width: usize);
    getter!(height: usize);

    pub fn new(width: usize, height: usize) -> World {
        World {
            entity_grid: Grid::new_filled_with(|| None, width, height),
            entities: Vec::with_capacity(width * height),
            width,
            height,
        }
    }

    pub fn add(&self, location: &Location, direction: &Direction) -> Location {
        self.entity_grid.add(location, direction)
    }

    pub fn get_entity(&self, location: &Location) -> Option<&Entity> {
        let guard = self.entity_grid.get(location);
        let entity = guard.clone();
        match entity {
            None => None,
            Some(id) => Some(&self.entities[id]),
        }
    }

    pub fn get_entity_mut(&mut self, location: &Location) -> Option<&mut Entity> {
        let guard = self.entity_grid.get(location);
        let entity = guard.clone();
        match entity {
            None => None,
            Some(id) => Some(&mut self.entities[id]),
        }
    }

    pub fn iter_entities_par(&self) -> Iter<Entity> {
        self.entities.par_iter()
    }

    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }

    pub fn move_entity(&mut self, location: &Location, direction: &Direction) -> Result<(), ()> {
        let new_location = self.add(location, direction);

        let mut source_guard = self.entity_grid.get(location);
        let mut target_guard = self.entity_grid.get(&new_location);
        if source_guard.is_none() || target_guard.is_some() {
            return Err(())
        }

        let id = source_guard.take().unwrap();
        self.entities.get_mut(id).unwrap().location = new_location.clone();
        target_guard.replace(id);
        return Ok(());
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        let mut guard = self.entity_grid.get(&entity.location);
        if guard.is_some() {
            return Err(());
        }

        let id = self.entities.len() as EntityId;
        self.entities.push(entity);
        guard.replace(id);
        return Ok(());
    }
}
