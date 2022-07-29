use bevy_ecs::prelude::*;

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
