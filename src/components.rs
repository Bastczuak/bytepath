use crate::{color::ColorGl, easings::EasingFunction, Timer};
use bevy_ecs::prelude::*;

#[derive(Component, Debug)]
pub struct Player {
  pub movement_speed: f32,
  pub rotation_speed: f32,
}

#[derive(Component, Debug)]
pub struct Projectile {
  pub movement_speed: f32,
}

#[derive(Component, Debug)]
pub struct DeadProjectile {
  pub timer: Timer,
}

#[derive(Component)]
pub struct ExplosionEffect {
  pub color: ColorGl,
}

#[derive(Component)]
pub struct TickEffect;

#[derive(Component)]
pub struct TrailEffect;

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Transform {
  pub rotation: glam::Quat,
  pub translation: glam::Vec3,
  pub center_rotation: glam::Quat,
}

impl Transform {
  pub fn mat4(&self) -> glam::Mat4 {
    glam::Mat4::from_rotation_translation(self.rotation, self.translation)
  }

  pub fn mat4_center(&self) -> glam::Mat4 {
    glam::Mat4::from_rotation_translation(self.center_rotation, self.translation)
  }
}

#[derive(Component, Debug)]
pub struct Interpolation {
  time: f32,
  duration: f32,
  begin_end: Vec<(f32, f32)>,
  repeating: bool,
}

impl Interpolation {
  pub fn new(begin_end: Vec<(f32, f32)>, duration: f32, repeating: bool) -> Self {
    Interpolation {
      time: 0.0,
      duration,
      begin_end,
      repeating,
    }
  }

  pub fn eval(&mut self, t: f32, easing_fn: EasingFunction) -> (Vec<f32>, bool) {
    self.time += t;
    let mut finished = false;
    if self.time >= self.duration {
      if self.repeating {
        self.time = 0.0;
      }
      finished = true;
    }
    (
      self
        .begin_end
        .iter()
        .map(|&(begin, end)| {
          let easing = (easing_fn)(self.time / self.duration);
          (1.0 - easing) * begin + easing * end
        })
        .collect(),
      finished,
    )
  }
}

#[derive(Component, Debug)]
pub struct Boost {
  pub max_boost: f32,
  pub boost: f32,
  pub cooldown: Option<f32>,
  pub inc_amount: f32,
  pub dec_amount: f32,
  pub cooldown_sec: Option<f32>,
}

impl Boost {
  pub fn is_empty(&self) -> bool {
    self.boost < 0.0
  }

  pub fn no_cooldown(&self) -> bool {
    self.cooldown.is_none()
  }

  pub fn can_boost(&self) -> bool {
    self.cooldown.is_none() && self.boost > 0.0
  }
}

impl Default for Boost {
  fn default() -> Self {
    Self {
      max_boost: 100.0,
      boost: 100.0,
      cooldown: None,
      inc_amount: 10.0,
      dec_amount: 50.0,
      cooldown_sec: Some(2.0),
    }
  }
}

#[derive(Component, Debug)]
pub struct AmmoPickup {
  pub movement_speed: f32,
  pub rotation_speed: f32,
  pub center_rotation_speed: f32,
  pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct BoostPickup {
  pub movement_speed: f32,
  pub movement_direction: f32,
  pub center_rotation_speed: f32,
  pub timer: Timer,
  pub visible: bool,
}
