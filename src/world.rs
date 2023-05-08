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
            location: Location::new(x, y, &world.entity_grid),
            facing: Direction::random(rng),
        }
    }

    pub fn determine_action(&self, world: &World, rng: &mut RngBuffer) -> Action {
        match rng.next() {
            roll if roll < 0.05 => Action::turn(self.location, Direction::random(rng)),
            roll if roll < 0.95 => Action::move_in_direction(self.location, self.facing),
            _ => Action::wait(self.location),
        }
    }

    pub fn step(&self, rng: &mut RngBuffer) {

        // TODO ...

    }
}

pub enum Action {
    Wait {
        location: Location
    },
    Move {
        from: Location,
        direction: Direction,
    },
    Turn {
        location: Location,
        facing: Direction,
    },
}

pub enum Outcome {
    Blocked,
    Wait { location: Location },
    Move { from: Location, direction: Direction },
    Turn { location: Location, facing: Direction },
}

impl Action {
    pub fn conflicting_directions(&self) -> Vec<Direction> {
        match self {
            Action::Wait { .. } => vec![],
            Action::Move { from: _, direction } => vec![*direction],
            Action::Turn { .. } => vec![],
        }
    }

    pub fn resolve(&self, world: &World, entity: &Entity) -> Outcome {
        match self {
            Action::Wait { location } => Outcome::Wait { location: *location },
            Action::Move { from, direction } => {
                let target_location = entity.location.plus(*direction, &world.entity_grid);
                match world.get_entity(target_location) {
                    Some(_) => Outcome::Blocked,
                    None => Outcome::Move { from: *from, direction: *direction },
                }
            }
            Action::Turn { location, facing } => Outcome::Turn { location: *location, facing: *facing },
        }
    }

    fn move_in_direction(location: Location, direction: Direction) -> Action {
        Action::Move {
            from: location,
            direction,
        }
    }

    fn turn(location: Location, facing: Direction) -> Action {
        Action::Turn { location, facing }
    }

    fn wait(location: Location) -> Action {
        Action::Wait { location }
    }
}

pub struct World {
    width: usize,
    height: usize,
    pub(crate) entity_grid: Grid<Option<Entity>>,
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

    pub fn get_entity(&self, location: Location) -> Option<&Entity> {
        self.entity_grid.get(location.coordinates()).as_ref()
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        match self.get_entity(entity.location) {
            Some(_) => Err(()),
            None => {
                match self.entity_grid.replace(entity.location.coordinates(), Some(entity)) {
                    None => Ok(()),
                    Some(_) => panic!("this space should be unoccupied")
                }
            }
        }
    }

    pub fn remove_entity(&mut self, location: Location) -> Option<Entity> {
        self.entity_grid.replace(location.coordinates(), None)
    }
}
