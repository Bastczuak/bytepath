use bevy_ecs::prelude::*;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct Position {
  pub x: f32,
  pub y: f32,
}

#[derive(Component, Debug)]
pub struct Angle {
  pub radians: f32,
  pub velocity: f32,
}

impl Default for Angle {
  fn default() -> Self {
    Self {
      radians: std::f32::consts::PI / 2.0,
      velocity: 1.66 * std::f32::consts::PI,
    }
  }
}

#[derive(Component, Debug)]
pub struct Velocity {
  pub base_x: f32,
  pub base_y: f32,
  pub x: f32,
  pub y: f32,
}

impl Velocity {
  pub fn new(value: f32) -> Self {
    Self {
      base_x: value,
      base_y: value,
      x: value,
      y: value,
    }
  }
}

#[derive(Component, Debug)]
pub struct Geometry {
  pub buffers_idx: usize,
}
