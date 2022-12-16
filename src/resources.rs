use crate::render::{gl::types::*, MyVertex};
use bevy_ecs::prelude::Resource;
use lyon::tessellation::{FillTessellator, StrokeTessellator, VertexBuffers};
use rand::rngs::SmallRng;
use sdl2::keyboard::Keycode;
use std::{
  collections::HashSet,
  marker::PhantomData,
  ops::{Deref, DerefMut},
  time::Duration,
};

#[derive(Debug, Resource)]
pub struct Camera {
  pub camera_pos: glam::Vec3,
  pub camera_front: glam::Vec3,
  pub camera_up: glam::Vec3,
  pub camera_zoom: glam::Vec3,
  pub camera_speed: f32,
}

impl Default for Camera {
  fn default() -> Self {
    Camera {
      camera_pos: glam::Vec3::new(0.0, 0.0, 3.0),
      camera_front: glam::Vec3::new(0.0, 0.0, -1.0),
      camera_up: glam::Vec3::new(0.0, 1.0, 0.0),
      camera_zoom: glam::Vec3::new(1.0, 1.0, 1.0),
      camera_speed: 2.5,
    }
  }
}

#[derive(Debug, Resource)]
pub struct Shake {
  pub is_shaking: bool,
  pub duration: f32,
  pub frequency: f32,
  pub amplitude: f32,
  pub time: f32,
  pub samples_x: Vec<f32>,
  pub samples_y: Vec<f32>,
}

#[derive(Debug, Resource)]
pub struct Flash {
  pub frame_cnt: u8,
  pub is_flashing: bool,
}

impl Default for Flash {
  fn default() -> Self {
    Self {
      frame_cnt: 4,
      is_flashing: false,
    }
  }
}

impl Default for Shake {
  fn default() -> Self {
    use rand::{Rng, SeedableRng};

    let duration = 0.6;
    let frequency = 60.0;
    let amplitude = 10.0;
    let sample_count = (duration * frequency) as usize;
    let mut rng = SmallRng::from_entropy();
    let samples_x = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
    let samples_y = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();

    Shake {
      is_shaking: false,
      duration,
      frequency,
      amplitude,
      time: 0.0,
      samples_x,
      samples_y,
    }
  }
}

pub type CircleGeometry = DrawBuffers<Circle>;
pub type QuadGeometry = DrawBuffers<Quad>;
pub type LineGeometry = DrawBuffers<Line>;

#[derive(Debug, Resource)]
pub struct DrawBuffers<Geometry> {
  pub vao: GLuint,
  pub vbo: GLuint,
  pub ebo: GLuint,
  pub vertex_buffer: VertexBuffers<MyVertex, u16>,
  _marker: PhantomData<Geometry>,
}

impl<T> DrawBuffers<T> {
  pub fn new(vao: GLuint, vbo: GLuint, ebo: GLuint) -> Self {
    Self {
      vao,
      vbo,
      ebo,
      vertex_buffer: VertexBuffers::new(),
      _marker: PhantomData::<T>::default(),
    }
  }
}

#[derive(Debug, Resource)]
pub struct Circle;

#[derive(Debug, Resource)]
pub struct Rectangle;

#[derive(Debug, Resource)]
pub struct Quad;

#[derive(Debug, Resource)]
pub struct Line;

#[derive(Debug, Resource)]
pub struct EntitySpawnTimer {
  pub projectile: Timer,
  pub tick_effect: Timer,
  pub ammo_pickup: Timer,
  pub boost_pickup: Timer,
}

impl Default for EntitySpawnTimer {
  fn default() -> Self {
    Self {
      projectile: Timer::from_seconds(0.25, true),
      tick_effect: Timer::from_seconds(5.0, true),
      ammo_pickup: Timer::from_seconds(1.0, true),
      boost_pickup: Timer::from_seconds(2.0, true),
    }
  }
}

impl EntitySpawnTimer {
  pub fn as_array(&mut self) -> [&mut Timer; 4] {
    [
      &mut self.projectile,
      &mut self.tick_effect,
      &mut self.ammo_pickup,
      &mut self.boost_pickup,
    ]
  }
}

#[derive(Debug, Default, Resource)]
pub struct Time {
  pub duration: Duration,
  pub slow_down_timer: Option<Duration>,
}

impl Deref for Time {
  type Target = Duration;

  fn deref(&self) -> &Self::Target {
    &self.duration
  }
}

impl DerefMut for Time {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.duration
  }
}

#[derive(Debug, Default, Resource)]
pub struct Timer {
  pub elapsed: f32,
  pub duration: f32,
  pub count: u8,
  pub finished: bool,
  pub repeating: bool,
}

impl Timer {
  pub fn from_seconds(seconds: f32, repeating: bool) -> Self {
    Self {
      duration: seconds,
      repeating,
      ..Default::default()
    }
  }

  pub fn tick(&mut self, delta: Duration) {
    self.elapsed = (self.elapsed + delta.as_secs_f32()).min(self.duration);

    if self.repeating && self.finished {
      self.reset();
    }

    if self.elapsed >= self.duration {
      self.finished = true;
      self.count += 1;
    }
  }

  pub fn reset(&mut self) {
    self.finished = false;
    self.elapsed = 0.0;
  }
}

#[derive(Resource)]
pub struct Fills(pub FillTessellator);

impl Deref for Fills {
  type Target = FillTessellator;
  fn deref(&self) -> &FillTessellator {
    &self.0
  }
}

impl DerefMut for Fills {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Resource)]
pub struct Strokes(pub StrokeTessellator);

impl Deref for Strokes {
  type Target = StrokeTessellator;
  fn deref(&self) -> &StrokeTessellator {
    &self.0
  }
}

impl DerefMut for Strokes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug, Resource)]
pub struct KeyCodes(pub HashSet<Keycode>);

impl Deref for KeyCodes {
  type Target = HashSet<Keycode>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for KeyCodes {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug, Resource)]
pub struct Randoms(pub SmallRng);

impl Deref for Randoms {
  type Target = SmallRng;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Randoms {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug, Resource)]
pub struct DurationWrapper(pub Duration);

impl Deref for DurationWrapper {
  type Target = Duration;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for DurationWrapper {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
