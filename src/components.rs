use rapier2d::math::Translation;
use specs::{prelude::*, Component};
use std::f32::consts::PI;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Transform {
  pub translation: Translation<f32>,
}

impl Transform {
  pub fn new(x: i16, y: i16) -> Self {
    Transform {
      translation: Translation::new(x as f32, y as f32),
    }
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Angle {
  pub radians: f32,
  pub velocity: f32,
}

impl Default for Angle {
  fn default() -> Self {
    Angle {
      radians: -PI / 2.0,
      velocity: 1.66 * PI,
    }
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Velocity {
  pub x: f32,
  pub y: f32,
}

impl Default for Velocity {
  fn default() -> Self {
    Velocity { x: 2.0, y: 2.0 }
  }
}
