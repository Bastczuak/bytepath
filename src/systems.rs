use crate::components::{Angle, Player, Transform, Velocity};
use crate::resources::{DeltaTick, Direction, MovementCommand, Shake};
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

      fn noise<'a>(samples: &'a [f32]) -> impl Fn(f32) -> f32 + 'a {
        move |n| {
          let n = n as usize;
          if n >= samples.len() {
            0.0
          } else {
            samples[n as usize]
          }
        }
      }
      let noise_x = noise(&self.samples_x);
      let noise_y = noise(&self.samples_y);
      let amplitude = |noise_fn: &dyn Fn(f32) -> f32| -> i32 {
        ((noise_fn(s0) + (s - s0) * (noise_fn(s1) - noise_fn(s0))) * k * 16.0) as i32
      };

      shake.x = amplitude(&noise_x);
      shake.y = amplitude(&noise_y);
    }
  }
}

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
  type SystemData = (
    Read<'a, Option<MovementCommand>>,
    Read<'a, DeltaTick>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Velocity>,
    WriteStorage<'a, Transform>,
    WriteStorage<'a, Angle>,
  );

  fn run(&mut self, (movement_command, ticks, players, velocities, mut transfroms, mut angles): Self::SystemData) {
    let movement_command = match &*movement_command {
      Some(movement_command) => movement_command,
      None => return,
    };

    for (_, velocity, transform, angle) in (&players, &velocities, &mut transfroms, &mut angles).join() {
      match movement_command {
        MovementCommand::Stop => {}
        MovementCommand::Move(direction) => match direction {
          Direction::Left => angle.radians -= angle.velocity * ticks.in_seconds(),
          Direction::Right => angle.radians += angle.velocity * ticks.in_seconds(),
        },
      }

      transform.translation.x += velocity.x * f32::cos(angle.radians);
      transform.translation.y += velocity.y * f32::sin(angle.radians);
    }
  }
}
