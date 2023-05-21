use rayon::prelude::*;
use rayon::slice::Iter;
use crate::entity::Entity;
use crate::rng_buffer;

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

    pub fn move_entity(&mut self, location: &Location, direction: &Direction) -> Result<(), ()> {
        let new_location = self.add(location, direction);

        let source = self.entity_grid.get_mut(location.index()).unwrap().take();
        let target = self.entity_grid.get_mut(new_location.index()).unwrap();

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

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North, Northeast, East, Southeast, South, Southwest, West, Northwest
}

impl Direction {
    pub fn x(&self) -> isize {
        match self {
            Direction::East | Direction::Northeast | Direction::Southeast => 1,
            Direction::West | Direction::Southwest | Direction::Northwest=> -1,
            Direction::North | Direction::South => 0,
        }
    }

    pub fn y(&self) -> isize {
        match self {
            Direction::South | Direction::Southeast | Direction::Southwest => 1,
            Direction::North | Direction::Northeast | Direction::Northwest => -1,
            Direction::East | Direction::West => 0,
        }
    }

    pub fn random() -> Direction {
        match (rng_buffer::next() * 8.0) as usize {
            0 => Direction::North,
            1 => Direction::Northeast,
            2 => Direction::East,
            3 => Direction::Southeast,
            4 => Direction::South,
            5 => Direction::Southwest,
            6 => Direction::West,
            7 => Direction::Northwest,
            _ => panic!("generated index should be in range")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Location {
    index: usize,
    x: usize,
    y: usize,
}

impl Location {
    getter!(x: usize);
    getter!(y: usize);
    getter!(index: usize);

    pub fn at(x: usize, y: usize, world: &World) -> Location {
        let x = x % world.width();
        let y = y % world.height();
        let index = x + world.width() * y;
        Location { x, y, index }
    }
}
