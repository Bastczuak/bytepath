use crate::components::Position;
use bevy_ecs::prelude::*;

pub fn print_position(query: Query<(Entity, &Position)>) {
  for (entity, position) in query.iter() {
    // println!("Entity {:?} is at position: x {}, y {}", entity, position.x, position.y);
  }
}
