use bevy_ecs::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Position {
  pub x: f32,
  pub y: f32,
}
