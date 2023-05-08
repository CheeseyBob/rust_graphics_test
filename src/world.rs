use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;
use crate::grid::{Direction, Grid, Location};

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

pub struct World {
    width: usize,
    height: usize,
    pub entity_grid: Grid<Option<Entity>>,
}

impl World {
    getter!(width: usize);
    getter!(height: usize);

    pub fn new(width: usize, height: usize) -> World {
        World {
            entity_grid: Grid::new_filled_with(|| None, width, height),
            width,
            height,
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for space in self.entity_grid.iter() {
            match space {
                Some(entity) => entity.draw(graphics),
                None => {}
            }
        }
    }

    pub fn load(&mut self, rng: &mut RngBuffer) {
        let mut count = 0;
        while count < 5_000 {
            let x = (rng.generate_next() * self.width as f64) as usize;
            let y = (rng.next() * self.height as f64) as usize;
            let entity = Entity::new(x, y, &self, rng);
            if self.place_entity(entity).is_ok() {
                count += 1;
            }
        }
    }

    pub fn get_entity(&self, location: &Location) -> Option<&Entity> {
        self.entity_grid.get(location).as_ref()
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        match self.get_entity(&entity.location) {
            Some(_) => Err(()),
            None => {
                match self.entity_grid.replace(&entity.location.clone(), Some(entity)) {
                    None => Ok(()),
                    Some(_) => panic!("this space should be unoccupied")
                }
            }
        }
    }

    pub fn remove_entity(&mut self, location: &Location) -> Option<Entity> {
        self.entity_grid.replace(location, None)
    }
}
