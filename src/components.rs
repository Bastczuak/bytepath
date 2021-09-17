use crate::easings::EasingFunction;
use sdl2::{pixels::Color, rect::Rect};
use specs::{prelude::*, Component};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ShootingEffect;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TickEffect;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TrailEffect;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Projectile;

#[derive(Component, Default, Copy, Clone)]
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
  pub base_x: f32,
  pub base_y: f32,
  pub x: f32,
  pub y: f32,
}

impl Velocity {
  pub fn new(value: f32) -> Self {
    Velocity {
      base_x: value,
      base_y: value,
      x: value,
      y: value,
    }
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
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

#[derive(Component, Copy, Clone)]
#[storage(DenseVecStorage)]
pub struct Sprite {
  pub z_index: u8,
  pub texture_idx: usize,
  pub rotation: f64,
  pub scale: f32,
  pub region: Rect,
}

impl Sprite {
  pub fn width(&self) -> f32 {
    self.region.width() as f32
  }

  pub fn height(&self) -> f32 {
    self.region.height() as f32
  }

  pub fn scaled_region_width(&self) -> u32 {
    (self.region.width() as f32 * self.scale) as u32
  }

  pub fn scaled_region_height(&self) -> u32 {
    (self.region.height() as f32 * self.scale) as u32
  }
}

impl Default for Sprite {
  fn default() -> Self {
    Self {
      texture_idx: 0,
      region: Rect::new(0, 0, 0, 0),
      rotation: 0.0,
      scale: 1.0,
      z_index: 1,
    }
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Animation {
  pub time: f32,
  pub frames: Vec<Sprite>,
}

impl Default for Animation {
  fn default() -> Self {
    Self {
      time: 0.0,
      frames: Vec::with_capacity(2),
    }
  }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct LineParticle {
  pub color: Color,
  pub width: f32,
  pub length: f32,
  pub x1: f32,
  pub y1: f32,
  pub x2: f32,
  pub y2: f32,
  pub time_to_live: f32,
}
