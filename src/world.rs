use std::slice::Iter;
use crate::graphics_window::{Color, GraphicsBuffer};
use crate::grid::{Direction, Grid, Location};
use crate::world::entity::Entity;

pub mod entity {
    use crate::graphics_window::{Color, GraphicsBuffer};
    use crate::grid::{Direction, Location};
    use crate::rng_buffer::RngBuffer;
    use crate::world::action::Action;
    use crate::world::World;

    pub struct Entity {
        pub location: Location,
        pub facing: Direction,
    }

    impl Entity {
        pub fn draw(&self, graphics: &mut GraphicsBuffer) {
            graphics.draw_pixel(self.location.x(), self.location.y(), Color::WHITE);
        }

        pub fn new(x: usize, y: usize, world: &World, rng: &mut RngBuffer) -> Entity {
            Entity {
                location: world.entity_grid.location_at(x, y),
                facing: Direction::random(rng),
            }
        }

        pub fn determine_action(&self, world: &World, rng: &mut RngBuffer) -> Action {
            match rng.next() {
                roll if roll < 0.05 => Action::Turn(Direction::random(rng)),
                roll if roll < 0.95 => Action::Move(self.facing),
                _ => Action::Wait,
            }
        }

        pub fn step(&self, rng: &mut RngBuffer) {

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

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);
        self.entities.iter().for_each(f!{entity -> entity.draw(graphics)});
    }

    pub fn get_entity(&self, location: &Location) -> Option<&Entity> {
        match self.entity_grid.get(location) {
            None => None,
            Some(id) => Some(&self.entities[*id]),
        }
    }

    pub fn get_entity_mut(&mut self, location: &Location) -> Option<&mut Entity> {
        match self.entity_grid.get(location) {
            None => None,
            Some(id) => Some(&mut self.entities[*id]),
        }
    }

    pub fn iter_entities(&self) -> Iter<'_, Entity> {
        self.entities.iter()
    }

    pub fn move_entity(&mut self, location: &Location, direction: &Direction) -> Result<(), ()> {
        let new_location = self.add(location, direction);
        let source = *self.entity_grid.get(location);
        let target = *self.entity_grid.get(&new_location);
        match (source, target) {
            (Some(id), None) => {
                let mut entity = &mut self.entities[id];
                entity.location = new_location;
                self.entity_grid.replace(location, target);
                self.entity_grid.replace(&new_location, source);
                Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        match self.get_entity(&entity.location) {
            Some(_) => Err(()),
            None => {
                let id = self.entities.len() as EntityId;
                let location = entity.location;
                self.entities.push(entity);
                match self.entity_grid.replace(&location, Some(id)) {
                    None => Ok(()),
                    Some(_) => panic!("this space should be unoccupied")
                }
            }
        }
    }
}
