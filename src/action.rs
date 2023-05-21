use crate::entity::Entity;
use crate::world::{Direction, World};

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
