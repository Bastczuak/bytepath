use crate::render::{gl::types::*, MyVertex};
use lyon::tessellation::VertexBuffers;
use std::marker::PhantomData;

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
pub struct Circle {}

#[derive(Debug)]
pub struct Rectangle {}
