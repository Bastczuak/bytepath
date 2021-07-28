use crate::{
  components::{
    Angle, Animation, Interpolation, LineParticle, Player, Position, Projectile, ShootingEffect, Sprite, Velocity,
  },
  easings::ease_out_sine,
  resources::{DeltaTick, GameEvents, GameEventsChannel, Shake},
  SCREEN_HEIGHT, SCREEN_WIDTH,
};
use rand::{Rng, SeedableRng};
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};
use specs::prelude::*;
use std::{collections::HashSet, f32::consts::PI};

#[derive(Default)]
pub struct ShakeSystem {
  duration: f32,
  frequency: f32,
  samples_x: Vec<f32>,
  samples_y: Vec<f32>,
  time: f32,
  is_shaking: bool,
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
          if n >= samples.len() { 0.0 } else { samples[n] }
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

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.duration = 1000.0;
    self.frequency = 40.0;
    let sample_count = ((self.duration / 1000.0) * self.frequency) as usize;
    let mut rng = rand::rngs::SmallRng::from_entropy();
    self.samples_x = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
    self.samples_y = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
  }
}

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, HashSet<Keycode>>,
    Read<'a, DeltaTick>,
    Write<'a, GameEventsChannel>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Velocity>,
    ReadStorage<'a, Sprite>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Angle>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, keycodes, ticks, mut events, players, velocities, sprites, mut positions, mut angles) = data;

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

      let sprite_offset_x = sprite.width() / 2.0;
      let sprite_offset_y = sprite.height() / 2.0;
      if (position.x - sprite_offset_x) < 0.0
        || (position.x + sprite_offset_x) > SCREEN_WIDTH as f32
        || (position.y - sprite_offset_y) < 0.0
        || (position.y + sprite_offset_y) > SCREEN_HEIGHT as f32
      {
        entities.delete(e).unwrap();
        events.single_write(GameEvents::PlayerDeath(*position));
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
      x = position.x + 0.5 * sprite.width() * f32::cos(angle.radians);
      y = position.y + 0.5 * sprite.width() * f32::sin(angle.radians);
      rotation = (angle.radians + PI / 4.0) * 180.0 / PI;
    }

    for (_, position, sprite, interpolation) in (&effects, &mut positions, &mut sprites, &mut interpolations).join() {
      position.x = x;
      position.y = y;
      sprite.rotation = rotation as f64;
      let value = interpolation.eval(8.0, 0.0, ticks.in_seconds());
      sprite.region = Rect::new(0, 0, value as u32, value as u32);
    }
  }
}

#[derive(Default)]
pub struct ProjectileSystem {
  spawn_time_s: Option<f32>,
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
                    + DISTANCE_MULTIPLIER * p_sprite.width() * f32::cos(p_angle.radians)
                    + (i as f32 * PROJECTILE_WIDTH).abs() * f32::cos(p_angle.radians + i as f32 * PI / 2.0),
                  y: p_pos.y
                    + DISTANCE_MULTIPLIER * p_sprite.height() * f32::sin(p_angle.radians)
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
                x: p_pos.x + DISTANCE_MULTIPLIER * p_sprite.width() * f32::cos(p_angle.radians),
                y: p_pos.y + DISTANCE_MULTIPLIER * p_sprite.height() * f32::sin(p_angle.radians),
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

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.spawn_time_s = Some(0.25);
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

#[derive(Default)]
pub struct PlayerDeathSystem {
  reader_id: Option<ReaderId<GameEvents>>,
  rng: Option<rand::rngs::SmallRng>,
  time_to_live: Option<f32>,
}

impl<'a> System<'a> for PlayerDeathSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, DeltaTick>,
    Write<'a, GameEventsChannel>,
    ReadStorage<'a, Angle>,
    WriteStorage<'a, Velocity>,
    WriteStorage<'a, LineParticle>,
    WriteStorage<'a, Interpolation>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, ticks, events, angels, mut velocities, mut particles, mut interpolations) = data;

    if let Some(mut time_to_live) = self.time_to_live.take() {
      time_to_live -= ticks.in_seconds();
      if time_to_live < 0.0 {
        for (e, _) in (&entities, &particles).join() {
          entities.delete(e).unwrap();
        }
      } else {
        self.time_to_live = Some(time_to_live);
      }
    }

    for (angle, velocity, particle, interpolation) in
      (&angels, &mut velocities, &mut particles, &mut interpolations).join()
    {
      particle.x1 += velocity.x * f32::cos(angle.radians) * ticks.in_seconds();
      particle.y1 += velocity.y * f32::sin(angle.radians) * ticks.in_seconds();
      particle.x2 = particle.x1 + particle.length * f32::cos(angle.radians);
      particle.y2 = particle.y1 + particle.length * f32::sin(angle.radians);

      let new_velocity = interpolation.eval(75.0, 0.0, ticks.in_seconds());
      velocity.x = new_velocity;
      velocity.y = new_velocity;
    }

    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      match event {
        GameEvents::PlayerDeath(pos) => {
          self.time_to_live = Some(1.5);
          let rng = self
            .rng
            .as_mut()
            .expect("rng Should not be None! Did you forget to initialize in setup()?");
          for _ in 0..12 {
            lazy
              .create_entity(&entities)
              .with(Angle {
                radians: rng.gen_range(0.0..2.0 * PI),
                velocity: 0.0,
              })
              .with(Velocity { x: 75.0, y: 75.0 })
              .with(Interpolation::new(1.5, ease_out_sine))
              .with(LineParticle {
                width: 2,
                color: Color::WHITE,
                length: rng.gen_range(2.0..3.0),
                x1: pos.x,
                y1: pos.y,
                x2: pos.x + 3.0 * f32::cos(PI / 4.0),
                y2: pos.y + 3.0 * f32::sin(PI / 4.0),
              })
              .build();
          }
        }
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(&world).register_reader());
    self.rng = Some(rand::rngs::SmallRng::from_entropy());
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id);
  }
}
