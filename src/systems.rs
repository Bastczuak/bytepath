use crate::resources::{DeltaTick, Shake};
use rand::Rng;
use specs::prelude::*;

pub struct ShakeSystem {
  duration: f32,
  frequency: f32,
  samples_x: Vec<f32>,
  samples_y: Vec<f32>,
  time: f32,
}

impl ShakeSystem {
  pub fn new() -> Self {
    let duration = 1000.0;
    let frequency = 40.0;
    let sample_count = ((duration / 1000.0) * frequency) as usize;
    let mut rng = rand::thread_rng();
    let samples_x = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
    let samples_y = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();

    ShakeSystem {
      duration,
      frequency,
      samples_x,
      samples_y,
      time: 0.0,
    }
  }
  //
  // fn update(&mut self, sdl_ticks: u32) {
  //   if let Some(ref mut time) = self.time {
  //     *time += sdl_ticks as f32;
  //     if *time > self.duration {
  //       self.time = None;
  //     }
  //   }
  // }
  //
  // fn amplitude(&mut self) -> Option<i32> {
  //   if let Some(time) = self.time {
  //     let s = time / 1000.0 * self.frequency;
  //     let s0 = f32::floor(s);
  //     let s1 = s0 + 1.0;
  //     let k = if time >= self.duration {
  //       0.0
  //     } else {
  //       (self.duration - time) / self.duration
  //     };
  //
  //     let noise = |n| {
  //       let n = n as usize;
  //       if n >= self.samples.len() {
  //         0.0
  //       } else {
  //         self.samples[n as usize]
  //       }
  //     };
  //     return Some(((noise(s0) + (s - s0) * (noise(s1) - noise(s0))) * k * 16.0) as i32);
  //   }
  //
  //   None
  // }
}

impl<'a> System<'a> for ShakeSystem {
  type SystemData = (Read<'a, DeltaTick>, Write<'a, Shake>);

  fn run(&mut self, (delta, mut shake): Self::SystemData) {
    if shake.is_shaking {
      self.time += delta.0 as f32;
      if self.time > self.duration {
        self.time = 0.0;
        shake.is_shaking = false;
      }

      let s = self.time / 1000.0 * self.frequency;
      let s0 = f32::floor(s);
      let s1 = s0 + 1.0;
      let k = if self.time >= self.duration {
        0.0
      } else {
        (self.duration - self.time) / self.duration
      };

      fn noise<'a>(samples: &'a[f32]) -> impl Fn(f32) -> f32 + 'a {
        return move |n| {
          let n = n as usize;
          if n >= samples.len() {
            0.0
          } else {
            samples[n as usize]
          }
        };
      }
      let noise_x = noise(&self.samples_x);
      let noise_y = noise(&self.samples_y);

      shake.x = ((noise_x(s0) + (s - s0) * (noise_x(s1) - noise_x(s0))) * k * 16.0) as i32;
      shake.y = ((noise_y(s0) + (s - s0) * (noise_y(s1) - noise_y(s0))) * k * 16.0) as i32;
    }
  }
}
