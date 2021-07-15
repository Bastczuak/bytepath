use crate::components::{
  Angle, Animation, Interpolation, Player, Position, Projectile, ShootingEffect, Sprite, Velocity,
};
use crate::resources::{DeltaTick, Shake};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use rand::Rng;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
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
    Entities<'a>,
    Read<'a, HashSet<Keycode>>,
    Read<'a, DeltaTick>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Velocity>,
    ReadStorage<'a, Sprite>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Angle>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, keycodes, ticks, players, velocities, sprites, mut positions, mut angles) = data;

    for (_, e, velocity, sprite, position, angle) in
      (&players, &entities, &velocities, &sprites, &mut positions, &mut angles).join()
    {
      for keycode in keycodes.iter() {
        match keycode {
          Keycode::D | Keycode::Left => angle.radians -= angle.velocity * ticks.in_seconds(),
          Keycode::A | Keycode::Right => angle.radians += angle.velocity * ticks.in_seconds(),
          _ => {}
        }
      }

      position.x += velocity.x * f32::cos(angle.radians);
      position.y += velocity.y * f32::sin(angle.radians);

      let sprite_offset_x = sprite.region.width() as f32 / 2.0;
      let sprite_offset_y = sprite.region.height() as f32 / 2.0;
      if (position.x - sprite_offset_x) < 0.0
        || (position.x + sprite_offset_x) > SCREEN_WIDTH as f32
        || (position.y - sprite_offset_y) < 0.0
        || (position.y + sprite_offset_y) > SCREEN_HEIGHT as f32
      {
        entities.delete(e).unwrap();
      }
    }
  }
}
pub struct ShootingSystem;

impl<'a> System<'a> for ShootingSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, DeltaTick>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Angle>,
    ReadStorage<'a, ShootingEffect>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Interpolation>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, ticks, players, angles, effects, mut positions, mut sprites, mut interpolations) = data;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut rotation = 0.0;
    let number_of_players = (&players, &entities).join().count();

    if number_of_players == 0 {
      // delete all shooting effects if there are no more players
      for (_, e) in (&effects, &entities).join() {
        entities.delete(e).unwrap();
      }
      return;
    }

    for (_, angle, position, sprite) in (&players, &angles, &mut positions, &sprites).join() {
      x = position.x + 0.5 * sprite.region.width() as f32 * f32::cos(angle.radians);
      y = position.y + 0.5 * sprite.region.width() as f32 * f32::sin(angle.radians);
      rotation = (angle.radians + PI / 4.0) * 180.0 / PI;
    }

    for (_, position, sprite, interpolation) in (&effects, &mut positions, &mut sprites, &mut interpolations).join() {
      position.x = x;
      position.y = y;
      sprite.rotation = rotation as f64;
      let value = interpolation.eval(ticks.in_seconds());
      sprite.region = Rect::new(0, 0, value as u32, value as u32);
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
    const PROJECTILE_HEIGHT: f32 = 8.0;
    const PROJECTILE_WIDTH: f32 = 8.0;

    let (entities, lazy, ticks, keycodes, players, projectiles, velocities, angles, sprites, mut positions) = data;

    for (_, e, velocity, angle, position) in (&projectiles, &entities, &velocities, &angles, &mut positions).join() {
      position.x += velocity.x * f32::cos(angle.radians);
      position.y += velocity.y * f32::sin(angle.radians);

      if position.x < 0.0 || position.x > SCREEN_WIDTH as f32 || position.y < 0.0 || position.y > SCREEN_HEIGHT as f32 {
        let x = if position.x < 0.0 {
          position.x + 1.5
        } else if position.x > SCREEN_WIDTH as f32 {
          position.x - 1.5
        } else {
          position.x
        };
        let y = if position.y < 0.0 {
          position.y + 1.5
        } else if position.y > SCREEN_HEIGHT as f32 {
          position.y - 1.5
        } else {
          position.y
        };
        let rotation = if position.x < 0.0 || position.x > SCREEN_WIDTH as f32 {
          90.0
        } else {
          0.0
        };

        lazy
          .create_entity(&entities)
          .with(Position { x, y })
          .with(Animation::new(vec![
            Sprite {
              texture_idx: 3,
              region: Rect::new(0, 0, 6, 3),
              rotation,
            },
            Sprite {
              texture_idx: 3,
              region: Rect::new(0, 3, 6, 3),
              rotation,
            },
          ]))
          .build();
      }

      if position.x < 0.0 || position.x > SCREEN_WIDTH as f32 || position.y < 0.0 || position.y > SCREEN_HEIGHT as f32 {
        entities.delete(e).unwrap();
      }
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
                    + DISTANCE_MULTIPLIER * p_sprite.region.width() as f32 * f32::cos(p_angle.radians)
                    + (i as f32 * PROJECTILE_WIDTH).abs() * f32::cos(p_angle.radians + i as f32 * PI / 2.0),
                  y: p_pos.y
                    + DISTANCE_MULTIPLIER * p_sprite.region.height() as f32 * f32::sin(p_angle.radians)
                    + (i as f32 * PROJECTILE_HEIGHT).abs() * f32::sin(p_angle.radians + i as f32 * PI / 2.0),
                })
                .with(*p_angle)
                .with(Velocity { x: 3.5, y: 3.5 })
                .with(Sprite {
                  texture_idx: 2,
                  region: Rect::new(0, 0, 8, 8),
                  rotation: (p_angle.radians * 180.0 / PI) as f64,
                })
                .build();
            }
          } else {
            lazy
              .create_entity(&entities)
              .with(Projectile)
              .with(Position {
                x: p_pos.x + DISTANCE_MULTIPLIER * p_sprite.region.width() as f32 * f32::cos(p_angle.radians),
                y: p_pos.y + DISTANCE_MULTIPLIER * p_sprite.region.height() as f32 * f32::sin(p_angle.radians),
              })
              .with(*p_angle)
              .with(Velocity { x: 3.5, y: 3.5 })
              .with(Sprite {
                texture_idx: 2,
                region: Rect::new(0, 0, 8, 8),
                rotation: (p_angle.radians * 180.0 / PI) as f64,
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

pub struct ProjectileDeathSystem;

impl<'a> System<'a> for ProjectileDeathSystem {
  type SystemData = (Entities<'a>, Read<'a, DeltaTick>, WriteStorage<'a, Animation>);

  fn run(&mut self, data: Self::SystemData) {
    let (entities, ticks, mut animations) = data;

    for (e, animation) in (&entities, &mut animations).join() {
      animation.time += ticks.in_seconds();

      if animation.time >= 0.25 {
        entities.delete(e).unwrap();
        continue;
      }

      if animation.time >= 0.1 {
        animation.frame_idx = 1;
      }
    }
  }
}
