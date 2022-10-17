use crate::render::{gl::types::*, MyVertex};
use lyon::tessellation::VertexBuffers;
use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
  time::Duration,
};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Shake {
  pub is_shaking: bool,
  pub duration: f32,
  pub frequency: f32,
  pub amplitude: f32,
  pub time: f32,
  pub samples_x: Vec<f32>,
  pub samples_y: Vec<f32>,
}

#[derive(Debug)]
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
    let mut rng = rand::rngs::SmallRng::from_entropy();
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Circle;

#[derive(Debug)]
pub struct Rectangle;

#[derive(Debug)]
pub struct Quad;

#[derive(Debug)]
pub struct Line;

#[derive(Debug)]
pub struct EntitySpawnTimer {
  pub projectile: Timer,
  pub tick_effect: Timer,
  pub ammo_pickup: Timer,
}

impl Default for EntitySpawnTimer {
  fn default() -> Self {
    Self {
      projectile: Timer::from_seconds(0.25, true),
      tick_effect: Timer::from_seconds(5.0, true),
      ammo_pickup: Timer::from_seconds(1.0, true),
    }
  }
}

impl EntitySpawnTimer {
  pub fn as_array(&mut self) -> [&mut Timer; 3] {
    [&mut self.projectile, &mut self.tick_effect, &mut self.ammo_pickup]
  }
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct Timer {
  pub elapsed: f32,
  pub duration: f32,
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
    }
  }

  pub fn reset(&mut self) {
    self.finished = false;
    self.elapsed = 0.0;
  }
}
