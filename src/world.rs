use std::ops::Deref;
use rayon::prelude::*;
use rayon::slice::{Iter, Windows};
use crate::graphics_window::Color;
use crate::grid::{Direction, Location};
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
                location: Location::at(x, y, &world),
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
                    let target_location = world.add(&entity.location, direction);
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
    entity_grid: Vec<Option<EntityId>>,
    entities: Vec<Entity>,
}

impl World {
    getter!(width: usize);
    getter!(height: usize);

    fn new_entity_grid(width: usize, height: usize) -> Vec<Option<EntityId>> {
        let mut entity_grid = Vec::with_capacity(width * height);
        entity_grid.resize_with(width * height, || None);
        return entity_grid;
    }

    pub fn new(width: usize, height: usize) -> World {
        World {
            entity_grid: Self::new_entity_grid(width, height),
            entities: Vec::with_capacity(width * height),
            width,
            height,
        }
    }

    pub fn add(&self, location: &Location, direction: &Direction) -> Location {
        let x = location.x() + self.width.checked_add_signed(direction.x()).expect("adding +1/-1 to width should not overflow");
        let y = location.y() + self.height.checked_add_signed(direction.y()).expect("adding +1/-1 to height should not overflow");
        Location::at(x, y, &self)
    }

    pub fn get_entity(&self, location: &Location) -> Option<&Entity> {
        let entity = self.entity_grid.get(location.index()).unwrap();
        match entity {
            None => None,
            Some(id) => Some(&self.entities[*id]),
        }
    }

    pub fn get_entity_mut(&mut self, location: &Location) -> Option<&mut Entity> {
        let entity = self.entity_grid.get(location.index()).unwrap();
        match entity {
            None => None,
            Some(id) => Some(&mut self.entities[*id]),
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

        let source = self.entity_grid.get_mut(location.index()).unwrap().take();
        let mut target = self.entity_grid.get_mut(new_location.index()).unwrap();

        if source.is_none() || target.is_some() {
            return Err(())
        }

        let id = source.unwrap();
        self.entities.get_mut(id).unwrap().location = new_location.clone();
        target.replace(id);
        return Ok(());
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        let index = entity.location.index();
        if self.entity_grid.get(index).unwrap().is_some() {
            return Err(());
        }

        let id = self.entities.len() as EntityId;
        self.entities.push(entity);

        self.entity_grid.get_mut(index)
            .unwrap()
            .replace(id);
        return Ok(());
    }
}
