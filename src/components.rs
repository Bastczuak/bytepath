use bevy_ecs::prelude::*;
use crate::easings::EasingFunction;

#[derive(Component, Debug)]
pub struct Player {
  pub movement_speed: f32,
  pub rotation_speed: f32,
}

#[derive(Component, Debug, Default)]
pub struct Transform {
  pub rotation: glam::Quat,
  pub translation: glam::Vec3,
}

impl Transform {
  pub fn mat4(&self) -> glam::Mat4 {
    glam::Mat4::from_rotation_translation(self.rotation, self.translation)
  }
}

#[derive(Component, Debug)]
pub struct Interpolation {
  time: f32,
  duration: f32,
  begin_end: Vec<(f32, f32)>,
}

impl Interpolation {
  pub fn new(begin_end: Vec<(f32, f32)>, duration: f32) -> Self {
    Interpolation {
      time: 0.0,
      duration,
      begin_end,
    }
  }

  pub fn eval(&mut self, t: f32, easing_fn: EasingFunction) -> (Vec<f32>, bool) {
    self.time += t;
    let mut finished = false;
    if self.time >= self.duration {
      self.time = 0.0;
      finished = true
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
