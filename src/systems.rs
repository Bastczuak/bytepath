use crate::components::{Angle, Interpolation, Player, Position, Projectile, ShootingEffect, Sprite, Velocity};
use crate::resources::{DeltaTick, Shake};
use rand::Rng;
use sdl2::keyboard::Keycode;
use specs::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;

pub struct ShakeSystem {
  duration: f32,
  frequency: f32,
  samples_x: Vec<f32>,
  samples_y: Vec<f32>,
  time: f32,
  is_shaking: bool,
}

impl Default for ShakeSystem {
  fn default() -> Self {
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
      is_shaking: false,
    }
  }
}

impl<'a> System<'a> for ShakeSystem {
  type SystemData = (Read<'a, HashSet<Keycode>>, Read<'a, DeltaTick>, Write<'a, Shake>);

  fn run(&mut self, (keycodes, ticks, mut shake): Self::SystemData) {
    if keycodes.contains(&Keycode::Space) {
      self.is_shaking = true;
    }

    if self.is_shaking {
      self.time += ticks.0 as f32;
      if self.time > self.duration {
        self.time = 0.0;
        self.is_shaking = false;
        return;
      }

      let s = self.time / 1000.0 * self.frequency;
      let s0 = f32::floor(s);
      let s1 = s0 + 1.0;
      let k = if self.time >= self.duration {
        0.0
      } else {
        (self.duration - self.time) / self.duration
      };

      fn noise(samples: &[f32]) -> impl Fn(f32) -> f32 + '_ {
        move |n| {
          let n = n as usize;
          if n >= samples.len() {
            0.0
          } else {
            samples[n]
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
    Read<'a, HashSet<Keycode>>,
    Read<'a, DeltaTick>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Velocity>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Angle>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (keycodes, ticks, players, velocities, mut positions, mut angles) = data;

    for (_, velocity, position, angle) in (&players, &velocities, &mut positions, &mut angles).join() {
      for keycode in keycodes.iter() {
        match keycode {
          Keycode::D | Keycode::Left => angle.radians -= angle.velocity * ticks.in_seconds(),
          Keycode::A | Keycode::Right => angle.radians += angle.velocity * ticks.in_seconds(),
          _ => {}
        }
      }

      position.x += velocity.x * f32::cos(angle.radians);
      position.y += velocity.y * f32::sin(angle.radians);
    }
  }
}
pub struct ShootingSystem;

impl<'a> System<'a> for ShootingSystem {
  type SystemData = (
    Read<'a, DeltaTick>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Angle>,
    ReadStorage<'a, ShootingEffect>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Interpolation>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (ticks, players, angles, effects, mut positions, mut sprites, mut interpolations) = data;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut rotation = 0.0;

    for (_, angle, position, sprite) in (&players, &angles, &mut positions, &sprites).join() {
      x = position.x + 0.5 * sprite.width as f32 * f32::cos(angle.radians);
      y = position.y + 0.5 * sprite.width as f32 * f32::sin(angle.radians);
      rotation = (angle.radians + PI / 4.0) * 180.0 / PI;
    }

    for (_, position, sprite, interpolation) in (&effects, &mut positions, &mut sprites, &mut interpolations).join() {
      position.x = x;
      position.y = y;
      sprite.rotation = rotation as f64;
      let value = interpolation.eval(ticks.in_seconds());
      sprite.width = value;
      sprite.height = value;
    }
  }
}

pub struct ProjectileSystem {
  spawn_time_s: Option<f32>,
}

impl Default for ProjectileSystem {
  fn default() -> Self {
    ProjectileSystem {
      spawn_time_s: Some(0.25),
    }
  }
}

impl<'a> System<'a> for ProjectileSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, DeltaTick>,
    Read<'a, HashSet<Keycode>>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Projectile>,
    ReadStorage<'a, Velocity>,
    ReadStorage<'a, Angle>,
    ReadStorage<'a, Sprite>,
    WriteStorage<'a, Position>,
  );

  fn run(&mut self, data: Self::SystemData) {
    const DISTANCE_MULTIPLIER: f32 = 0.8;
    const PROJECTILE_HEIGHT: f32 = 6.0;
    const PROJECTILE_WIDTH: f32 = 6.0;

    let (entities, lazy, ticks, keycodes, players, projectiles, velocities, angles, sprites, mut positions) = data;

    for (_, velocity, angle, position) in (&projectiles, &velocities, &angles, &mut positions).join() {
      position.x += velocity.x * f32::cos(angle.radians);
      position.y += velocity.y * f32::sin(angle.radians);
    }

    if let Some(mut timer) = self.spawn_time_s.take() {
      timer -= ticks.in_seconds();
      if timer <= 0.0 {
        for (_, p_angle, p_pos, p_sprite) in (&players, &angles, &positions, &sprites).join() {
          if keycodes.contains(&Keycode::F1) {
            for i in -1..2 {
              lazy
                .create_entity(&entities)
                .with(Projectile)
                .with(Position {
                  x: p_pos.x
                    + DISTANCE_MULTIPLIER * p_sprite.width as f32 * f32::cos(p_angle.radians)
                    + (i as f32 * PROJECTILE_WIDTH).abs() * f32::cos(p_angle.radians + i as f32 * PI / 2.0),
                  y: p_pos.y
                    + DISTANCE_MULTIPLIER * p_sprite.height as f32 * f32::sin(p_angle.radians)
                    + (i as f32 * PROJECTILE_HEIGHT).abs() * f32::sin(p_angle.radians + i as f32 * PI / 2.0),
                })
                .with(p_angle.clone())
                .with(Velocity { x: 3.5, y: 3.5 })
                .with(Sprite {
                  position: 2,
                  width: PROJECTILE_WIDTH,
                  height: PROJECTILE_HEIGHT,
                  rotation: 0.0,
                })
                .build();
            }
          } else {
            lazy
              .create_entity(&entities)
              .with(Projectile)
              .with(Position {
                x: p_pos.x + DISTANCE_MULTIPLIER * p_sprite.width as f32 * f32::cos(p_angle.radians),
                y: p_pos.y + DISTANCE_MULTIPLIER * p_sprite.height as f32 * f32::sin(p_angle.radians),
              })
              .with(p_angle.clone())
              .with(Velocity { x: 3.5, y: 3.5 })
              .with(Sprite {
                position: 2,
                width: PROJECTILE_WIDTH,
                height: PROJECTILE_HEIGHT,
                rotation: 0.0,
              })
              .build();
          }
        }
        self.spawn_time_s.replace(0.25);
      } else {
        self.spawn_time_s.replace(timer);
      }
    }
  }
}
