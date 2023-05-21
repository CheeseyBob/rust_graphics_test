use crate::graphics_window::Color;
use crate::rng_buffer;
use crate::action::Action;
use crate::world::{Direction, Location, World};

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
