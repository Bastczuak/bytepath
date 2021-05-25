use crate::easings::EasingFunction;
use sdl2::rect::Rect;
use specs::{prelude::*, Component};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ShootingEffect;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Projectile;

#[derive(Component, Default)]
#[storage(DenseVecStorage)]
pub struct Position {
  pub x: f32,
  pub y: f32,
}

#[derive(Component, Copy, Clone)]
#[storage(DenseVecStorage)]
pub struct Angle {
  pub radians: f32,
  pub velocity: f32,
}

impl Default for Angle {
  fn default() -> Self {
    Angle {
      radians: -std::f32::consts::PI / 2.0,
      velocity: 1.66 * std::f32::consts::PI,
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

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Interpolation {
  time: f32,
  duration: f32,
  easing_fn: EasingFunction,
  v0: f32,
  v1: f32,
}

impl Interpolation {
  pub fn new(v0: f32, v1: f32, duration: f32, easing_fn: EasingFunction) -> Self {
    Interpolation {
      time: 0.0,
      duration,
      easing_fn,
      v0,
      v1,
    }
  }

  pub fn eval(&mut self, t: f32) -> f32 {
    self.time += t;
    if self.time >= self.duration {
      self.time = 0.0;
      return self.v0;
    }
    let easing = (self.easing_fn)(self.time / self.duration);
    (1.0 - easing) * self.v0 + easing * self.v1
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Sprite {
  pub position: usize,
  pub width: f32,
  pub height: f32,
  pub rotation: f64,
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Animation {
  pub time: f32,
  pub position: usize,
  pub width: f32,
  pub height: f32,
  pub rotation: f64,
  pub current_frame: Option<Rect>,
  pub first_frame: Rect,
  pub second_frame: Rect,
}

impl Animation {
  pub fn with_rotation(rotation: f64) -> Self {
    Animation {
      time: 0.0,
      position: 0,
      width: 0.0,
      height: 0.0,
      rotation,
      current_frame: None,
      first_frame: Rect::new(0, 0, 0, 0),
      second_frame: Rect::new(0, 0, 0, 0),
    }
  }
}

impl Default for Animation {
  fn default() -> Self {
    Animation {
      time: 0.0,
      position: 0,
      width: 0.0,
      height: 0.0,
      rotation: 0.0,
      current_frame: None,
      first_frame: Rect::new(0, 0, 0, 0),
      second_frame: Rect::new(0, 0, 0, 0),
    }
  }
}
