use crate::graphics_window::{Color, GraphicsBuffer};
use crate::rng_buffer::RngBuffer;

pub struct Entity {
    x: usize,
    y: usize,
}

impl Entity {
    fn new(x: usize, y: usize) -> Entity {
        Entity { x, y }
    }

    fn get_location(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}

pub struct World {
    entities: Vec<Entity>,
    width: usize,
    height: usize,
    rng_buffer: RngBuffer,
}

impl World {
    pub fn new(width: usize, height: usize) -> World {
        World {
            entities: Vec::with_capacity(100),
            width,
            height,
            rng_buffer: RngBuffer::init_new(100_000, ())
        }
    }

    pub fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for object in &self.entities {
            graphics.draw_pixel(object.x, object.y, Color::WHITE);
        }
    }

    pub fn step(&mut self) {
        if self.entities.len() < 1000 {
            match self.place_entity(Entity::new(100, 100)) {
                Ok(_) => {}
                Err(_) => {}
            }
        }

        for entity in &mut self.entities {
            let (x, y) = entity.get_location();
            let (dx, dy) = (1, 0);
            let (x, y) = (x + dx, y + dy);
            let is_occupied = self.is_occupied(x, y);
            if !is_occupied {
                //entity.x = x;
                //entity.y = y;
            }

        }
    }

    fn is_occupied(&self, x: usize, y: usize) -> bool {
        for entity in &self.entities {
            if entity.x == x && entity.y == y {
                return true;
            }
        }
        return false;
    }

    fn move_entity(&mut self, entity: &mut Entity, x: usize, y: usize) -> Result<(), ()> {
        if self.is_occupied(x, y) {
            return Err(());
        }
        entity.x = x;
        entity.y = y;
        return Ok(());
    }

    fn place_entity(&mut self, entity: Entity) -> Result<(), ()> {
        if self.is_occupied(entity.x, entity.y) {
            return Err(());
        }

        self.entities.push(entity);
        return Ok(());
    }

    fn remove_entity(&mut self, x: usize, y: usize) -> Option<Entity> { // BAD
        let mut index : Option<usize> = None;
        for i in 0..self.entities.len() {
            if self.entities[i].get_location() == (x, y) {
                index = i;
                break;
            }
        }
        match index {
            None => None
            Some(index) => Some(self.entities.remove(index))
        }
    }
}