use crate::{
  components::{
    Ammunition, Angle, Animation, Boost, Interpolation, LineParticle, Player, Position, Projectile, ShootingEffect,
    Sprite, TickEffect, TrailEffect, Velocity,
  },
  easings::{ease_in_out_cubic, linear},
  environment::{Z_INDEX_BOOST_TRAIL, Z_INDEX_PLAYER},
  resources::{
    Flash, GameEvents,
    GameEvents::{PlayerDeath, PlayerSpawn},
    GameEventsChannel, Shake,
  },
  SCREEN_HEIGHT, SCREEN_WIDTH,
};
use rand::{Rng, SeedableRng};
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};
use specs::prelude::*;
use std::{collections::HashSet, f32::consts::PI, time::Duration};

#[derive(Default)]
pub struct AmmunitionSystem {
  rng: Option<rand::rngs::SmallRng>,
}

impl<'a> System<'a> for AmmunitionSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, HashSet<Keycode>>,
    Read<'a, Duration>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Ammunition>,
    ReadStorage<'a, Angle>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Velocity>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, keycodes, time, players, ammunition, angles, mut sprites, mut positions, mut velocities) =
      data;

    for keycode in keycodes.iter() {
      if keycode == &Keycode::S {
        let rng = self
          .rng
          .as_mut()
          .expect("rng Should not be None! Did you forget to initialize in setup()?");
        lazy
          .create_entity(&entities)
          .with(Ammunition)
          .with(Sprite {
            texture_idx: 6,
            region: Rect::new(0, 0, 6, 6),
            ..Default::default()
          })
          .with(Position {
            x: rng.gen_range(0.0..SCREEN_WIDTH as f32),
            y: rng.gen_range(0.0..SCREEN_HEIGHT as f32),
          })
          .with(Velocity::new(rng.gen_range(10.0..20.0)))
          .with(Angle {
            radians: rng.gen_range(0.0..2.0) * PI,
            velocity: rng.gen_range(-4.0..4.0) * PI,
          })
          .build();
      }
    }

    fn normalize((x, y): (f32, f32)) -> (f32, f32) {
      let length = f32::sqrt(x * x + y * y);
      (x / length, y / length)
    }

    let player_pos = (&players, &positions)
      .join()
      .collect::<Vec<_>>()
      .get(0)
      .map(|(_, pos)| **pos);

    for (_, e, angle, sprite, position, velocity) in (
      &ammunition,
      &entities,
      &angles,
      &mut sprites,
      &mut positions,
      &mut velocities,
    )
      .join()
    {
      sprite.rotation += ((angle.velocity * 180.0 / PI) * time.as_secs_f32()) as f64;

      if let Some(player_pos) = player_pos {
        let projectile_heading = normalize((velocity.x, velocity.y));
        let angle = f32::atan2(player_pos.y - position.y, player_pos.x - position.x);
        let to_target_heading = normalize((f32::cos(angle), f32::sin(angle)));
        let final_heading = normalize((
          projectile_heading.0 + 0.1 * to_target_heading.0,
          projectile_heading.1 + 0.1 * to_target_heading.1,
        ));
        velocity.x = velocity.base_x * final_heading.0;
        velocity.y = velocity.base_y * final_heading.1;
      }

      position.x += velocity.x * time.as_secs_f32() * f32::cos(angle.radians);
      position.y += velocity.y * time.as_secs_f32() * f32::sin(angle.radians);

      let sprite_offset_x = sprite.width() / 2.0;
      let sprite_offset_y = sprite.height() / 2.0;
      if (position.x + sprite_offset_x) < 0.0
        || (position.x - sprite_offset_x) > SCREEN_WIDTH as f32
        || (position.y + sprite_offset_y) < 0.0
        || (position.y - sprite_offset_y) > SCREEN_HEIGHT as f32
      {
        entities.delete(e).unwrap();
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.rng = Some(rand::rngs::SmallRng::from_entropy());
  }
}

#[derive(Default)]
pub struct TrailEffectSystem {
  timer: Option<f32>,
  rng: Option<rand::rngs::SmallRng>,
}

impl<'a> System<'a> for TrailEffectSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
    Read<'a, HashSet<Keycode>>,
    WriteStorage<'a, Interpolation>,
    WriteStorage<'a, Animation>,
    WriteStorage<'a, Sprite>,
    ReadStorage<'a, Position>,
    ReadStorage<'a, Angle>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, TrailEffect>,
    ReadStorage<'a, Boost>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (
      entities,
      lazy,
      time,
      keycodes,
      mut interpolations,
      mut animations,
      mut sprites,
      positions,
      angles,
      players,
      effects,
      boosts,
    ) = data;

    let player_boost = (&players, &boosts)
      .join()
      .collect::<Vec<_>>()
      .get(0)
      .map(|(_, boost)| *boost);

    for (_, e, interpolation, animation, sprite) in
    (&effects, &entities, &mut interpolations, &mut animations, &mut sprites).join()
    {
      let (values, finished) = interpolation.eval(time.as_secs_f32(), linear);

      if let Some(player_boost) = player_boost {
        if (keycodes.contains(&Keycode::Up) || keycodes.contains(&Keycode::Down)) && player_boost.can_boost() {
          // TODO: The Sprite struct is copied every single frame but it should just get toggled
          *sprite = animation.frames[1];
        } else {
          // TODO: The Sprite struct is copied every single frame but it should just get toggled
          *sprite = animation.frames[0];
        }
      } else {
        // TODO: The Sprite struct is copied every single frame but it should just get toggled
        *sprite = animation.frames[0];
      }

      sprite.scale = values[0];
      if finished {
        entities.delete(e).unwrap();
      }
    }

    // don't spawn new effect if there is no player
    if (&players, &entities).join().count() == 0 {
      self.timer.replace(0.01);
      return;
    }

    if let Some(mut timer) = self.timer.take() {
      timer -= time.as_secs_f32();
      if timer < 0.0 {
        let rng = self
          .rng
          .as_mut()
          .expect("rng Should not be None! Did you forget to initialize in setup()?");
        let mut x = 0.0;
        let mut y = 0.0;
        let scale = rng.gen_range(0.25..0.35);
        let width = 32;
        let height = 32;
        for (_, pos, angle) in (&players, &positions, &angles).join() {
          x = pos.x - 0.5 * width as f32 * f32::cos(angle.radians);
          y = pos.y - 0.5 * height as f32 * f32::sin(angle.radians);
        }
        lazy
          .create_entity(&entities)
          .with(TrailEffect)
          .with(Position { x, y })
          .with(Sprite::default())
          .with(Animation {
            frames: vec![
              Sprite {
                texture_idx: 5,
                region: Rect::new(0, 0, width, height),
                scale,
                z_index: Z_INDEX_BOOST_TRAIL,
                ..Default::default()
              },
              Sprite {
                texture_idx: 5,
                region: Rect::new(32, 0, width, height),
                scale,
                z_index: Z_INDEX_BOOST_TRAIL,
                ..Default::default()
              },
            ],
            ..Default::default()
          })
          .with(Interpolation::new(vec![(scale, 0.0)], rng.gen_range(0.15..0.25)))
          .build();
        self.timer.replace(0.01);
      } else {
        self.timer.replace(timer);
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.timer = Some(0.01);
    self.rng = Some(rand::rngs::SmallRng::from_entropy());
  }
}

#[derive(Default)]
pub struct TickEffectSystem {
  timer: Option<f32>,
}

impl<'a> System<'a> for TickEffectSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Interpolation>,
    WriteStorage<'a, Sprite>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, TickEffect>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, time, mut positions, mut interpolations, mut sprites, players, effects) = data;

    // don't process any effects if there is no player entity and make sure to clean up existing ones.
    if (&players, &entities).join().count() == 0 {
      for (_, e) in (&effects, &entities).join() {
        entities.delete(e).unwrap();
        self.timer.replace(5.0);
      }
      return;
    }

    let mut x = 0.0;
    let mut y = 0.0;
    for (_, pos) in (&players, &positions).join() {
      x = pos.x;
      y = pos.y;
    }

    for (e, _, pos, interpolation, sprite) in
      (&entities, &effects, &mut positions, &mut interpolations, &mut sprites).join()
    {
      let (values, finished) = interpolation.eval(time.as_secs_f32(), ease_in_out_cubic);
      pos.x = x;
      pos.y = y - values[1];
      sprite.region = Rect::new(0, 0, sprite.region.width(), values[0] as u32);
      if finished {
        entities.delete(e).unwrap();
      }
    }

    if let Some(mut timer) = self.timer.take() {
      timer -= time.as_secs_f32();
      if timer < 0.0 {
        lazy
          .create_entity(&entities)
          .with(TickEffect)
          .with(Position { x, y })
          .with(Sprite {
            texture_idx: 4,
            region: Rect::new(0, 0, 48, 32),
            ..Default::default()
          })
          .with(Interpolation::new(vec![(32.0, 0.0), (0.0, 32.0)], 0.1))
          .build();
        self.timer.replace(5.0);
      } else {
        self.timer.replace(timer);
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.timer = Some(5.0);
  }
}

#[derive(Default)]
pub struct FlashSystem {
  reader_id: Option<ReaderId<GameEvents>>,
}

impl<'a> System<'a> for FlashSystem {
  type SystemData = (Write<'a, Flash>, Write<'a, GameEventsChannel>);

  fn run(&mut self, (mut flash, events): Self::SystemData) {
    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      if let PlayerDeath(_) = event {
        flash.0 = 4;
      }
    }

    if flash.0 > 0 {
      flash.0 -= 1;
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(world).register_reader());
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id)
  }
}

#[derive(Default)]
pub struct ShakeSystem {
  duration: f32,
  frequency: f32,
  amplitude: f32,
  samples_x: Vec<f32>,
  samples_y: Vec<f32>,
  time: f32,
  is_shaking: bool,
  reader_id: Option<ReaderId<GameEvents>>,
}

impl<'a> System<'a> for ShakeSystem {
  type SystemData = (Read<'a, Duration>, Write<'a, Shake>, Write<'a, GameEventsChannel>);

  fn run(&mut self, (time, mut shake, events): Self::SystemData) {
    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      if let PlayerDeath(_) = event {
        self.is_shaking = true;
      }
    }

    if self.is_shaking {
      self.time += time.as_secs_f32();
      if self.time > self.duration {
        self.time = 0.0;
        self.is_shaking = false;
        return;
      }

      let s = self.time * self.frequency;
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
        ((noise_fn(s0) + (s - s0) * (noise_fn(s1) - noise_fn(s0))) * k * self.amplitude) as i32
      };

      shake.x = amplitude(&noise_x);
      shake.y = amplitude(&noise_y);
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.duration = 0.4;
    self.frequency = 60.0;
    self.amplitude = 6.0;
    let sample_count = (self.duration * self.frequency) as usize;
    let mut rng = rand::rngs::SmallRng::from_entropy();
    self.samples_x = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
    self.samples_y = (0..sample_count).map(|_| rng.gen_range(0.0..1.0) * 2.0 - 1.0).collect();
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(world).register_reader());
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id);
  }
}

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, HashSet<Keycode>>,
    Read<'a, Duration>,
    ReadStorage<'a, Player>,
    Write<'a, GameEventsChannel>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Velocity>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Angle>,
    WriteStorage<'a, Boost>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (
      entities,
      lazy,
      keycodes,
      time,
      players,
      mut events,
      mut sprites,
      mut velocities,
      mut positions,
      mut angles,
      mut boosts,
    ) = data;

    for (_, e, sprite, velocity, position, angle, boost) in (
      &players,
      &entities,
      &mut sprites,
      &mut velocities,
      &mut positions,
      &mut angles,
      &mut boosts,
    )
      .join()
    {
      for keycode in keycodes.iter() {
        match keycode {
          Keycode::Left => angle.radians -= angle.velocity * time.as_secs_f32(),
          Keycode::Right => angle.radians += angle.velocity * time.as_secs_f32(),
          Keycode::Up => {
            if boost.can_boost() {
              velocity.x *= 1.5;
              velocity.y *= 1.5;
              boost.boost -= boost.dec_amount * time.as_secs_f32();
            }
          }
          Keycode::Down => {
            if boost.can_boost() {
              velocity.x *= 0.5;
              velocity.y *= 0.5;
              boost.boost -= boost.dec_amount * time.as_secs_f32();
            }
          }
          Keycode::D => {
            entities.delete(e).unwrap();
            events.single_write(GameEvents::PlayerDeath(*position));
          }
          _ => {}
        }
      }

      if boost.is_empty() && boost.no_cooldown() {
        boost.cooldown = boost.cooldown_sec;
      } else if let Some(mut cooldown) = boost.cooldown.take() {
        cooldown -= time.as_secs_f32();
        if cooldown > 0.0 {
          boost.cooldown.replace(cooldown);
        }
      }

      sprite.rotation = (angle.radians * 180.0 / PI) as f64;
      position.x += velocity.x * time.as_secs_f32() * f32::cos(angle.radians);
      position.y += velocity.y * time.as_secs_f32() * f32::sin(angle.radians);
      velocity.x = velocity.base_x;
      velocity.y = velocity.base_y;
      boost.boost = boost.max_boost.min(boost.boost + boost.inc_amount * time.as_secs_f32());

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

    for keycode in keycodes.iter() {
      if let Keycode::S = keycode {
        let number_of_players = (&players, &entities).join().count();
        if number_of_players == 0 {
          lazy
            .create_entity(&entities)
            .with(Player)
            .with(Position {
              x: SCREEN_WIDTH as f32 / 2.0,
              y: SCREEN_HEIGHT as f32 / 2.0,
            })
            .with(Angle::default())
            .with(Velocity::new(100.0))
            .with(Sprite {
              texture_idx: 0,
              region: Rect::new(0, 0, 32, 32),
              z_index: Z_INDEX_PLAYER,
              ..Default::default()
            })
            .with(Boost::default())
            .build();
          events.single_write(PlayerSpawn);
        }
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    world
      .create_entity()
      .with(Player)
      .with(Position {
        x: SCREEN_WIDTH as f32 / 2.0,
        y: SCREEN_HEIGHT as f32 / 2.0,
      })
      .with(Angle::default())
      .with(Velocity::new(100.0))
      .with(Sprite {
        texture_idx: 0,
        region: Rect::new(0, 0, 32, 32),
        z_index: Z_INDEX_PLAYER,
        ..Default::default()
      })
      .with(Boost::default())
      .build();
  }
}

#[derive(Default)]
pub struct ShootingSystem {
  reader_id: Option<ReaderId<GameEvents>>,
}

impl<'a> System<'a> for ShootingSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
    Write<'a, GameEventsChannel>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Angle>,
    ReadStorage<'a, ShootingEffect>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Sprite>,
    WriteStorage<'a, Interpolation>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, time, events, players, angles, effects, mut positions, mut sprites, mut interpolations) = data;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut rotation = 0.0;

    // delete all shooting effects if there are no more players
    if (&players, &entities).join().count() == 0 {
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
      let (values, _) = interpolation.eval(time.as_secs_f32(), ease_in_out_cubic);
      sprite.scale = values[0];
    }

    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      if let PlayerSpawn = event {
        lazy
          .create_entity(&entities)
          .with(ShootingEffect)
          .with(Position {
            x: SCREEN_WIDTH as f32 / 2.0,
            y: SCREEN_HEIGHT as f32 / 2.0,
          })
          .with(Sprite {
            texture_idx: 1,
            region: Rect::new(0, 0, 8, 8),
            rotation: 45.0,
            ..Default::default()
          })
          .with(Interpolation::new(vec![(1.0, 0.0)], 0.2))
          .build();
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(world).register_reader());
    world
      .create_entity()
      .with(ShootingEffect)
      .with(Position {
        x: SCREEN_WIDTH as f32 / 2.0,
        y: SCREEN_HEIGHT as f32 / 2.0,
      })
      .with(Sprite {
        texture_idx: 1,
        region: Rect::new(0, 0, 8, 8),
        rotation: 45.0,
        ..Default::default()
      })
      .with(Interpolation::new(vec![(1.0, 0.0)], 0.2))
      .build();
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id);
  }
}

#[derive(Default)]
pub struct ProjectileSystem {
  spawn_time_s: Option<f32>,
}

impl<'a> System<'a> for ProjectileSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Write<'a, GameEventsChannel>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
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

    let (entities, mut events, lazy, time, keycodes, players, projectiles, velocities, angles, sprites, mut positions) =
      data;

    for (_, e, velocity, angle, position) in (&projectiles, &entities, &velocities, &angles, &mut positions).join() {
      position.x += velocity.x * time.as_secs_f32() * f32::cos(angle.radians);
      position.y += velocity.y * time.as_secs_f32() * f32::sin(angle.radians);

      if position.x < 0.0 || position.x > SCREEN_WIDTH as f32 || position.y < 0.0 || position.y > SCREEN_HEIGHT as f32 {
        entities.delete(e).unwrap();
        events.single_write(GameEvents::ProjectileDeath(*position));
      }
    }

    if let Some(mut timer) = self.spawn_time_s.take() {
      timer -= time.as_secs_f32();
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
                .with(Velocity::new(150.0))
                .with(Sprite {
                  texture_idx: 2,
                  region: Rect::new(0, 0, 8, 8),
                  rotation: (p_angle.radians * 180.0 / PI) as f64,
                  ..Default::default()
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
              .with(Velocity::new(150.0))
              .with(Sprite {
                texture_idx: 2,
                region: Rect::new(0, 0, 8, 8),
                rotation: (p_angle.radians * 180.0 / PI) as f64,
                ..Default::default()
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

#[derive(Default)]
pub struct ProjectileDeathSystem {
  reader_id: Option<ReaderId<GameEvents>>,
}

impl<'a> System<'a> for ProjectileDeathSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Write<'a, GameEventsChannel>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
    WriteStorage<'a, Animation>,
    WriteStorage<'a, Sprite>,
    ReadStorage<'a, Projectile>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, events, lazy, time, mut animations, mut sprites, projectiles) = data;

    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      if let GameEvents::ProjectileDeath(pos) = event {
        let x = if pos.x < 0.0 {
          pos.x + 1.5
        } else if pos.x > SCREEN_WIDTH as f32 {
          pos.x - 1.5
        } else {
          pos.x
        };
        let y = if pos.y < 0.0 {
          pos.y + 1.5
        } else if pos.y > SCREEN_HEIGHT as f32 {
          pos.y - 1.5
        } else {
          pos.y
        };
        let rotation = if pos.x < 0.0 || pos.x > SCREEN_WIDTH as f32 {
          90.0
        } else {
          0.0
        };

        lazy
          .create_entity(&entities)
          .with(Projectile)
          .with(Position { x, y })
          .with(Sprite {
            texture_idx: 3,
            region: Rect::new(0, 0, 6, 3),
            rotation,
            ..Default::default()
          })
          .with(Animation {
            frames: vec![
              Sprite {
                texture_idx: 3,
                region: Rect::new(0, 0, 6, 3),
                rotation,
                ..Default::default()
              },
              Sprite {
                texture_idx: 3,
                region: Rect::new(0, 3, 6, 3),
                rotation,
                ..Default::default()
              },
            ],
            ..Default::default()
          })
          .build();
      }
    }
    for (_, e, animation, sprite) in (&projectiles, &entities, &mut animations, &mut sprites).join() {
      animation.time += time.as_secs_f32();

      if animation.time >= 0.25 {
        entities.delete(e).unwrap();
        continue;
      }

      if animation.time >= 0.1 {
        *sprite = animation.frames[1];
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(world).register_reader());
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id)
  }
}

#[derive(Default)]
pub struct PlayerDeathSystem {
  reader_id: Option<ReaderId<GameEvents>>,
  rng: Option<rand::rngs::SmallRng>,
}

impl<'a> System<'a> for PlayerDeathSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, Duration>,
    Write<'a, GameEventsChannel>,
    ReadStorage<'a, Angle>,
    WriteStorage<'a, Velocity>,
    WriteStorage<'a, LineParticle>,
    WriteStorage<'a, Interpolation>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, time, events, angels, mut velocities, mut particles, mut interpolations) = data;

    for (e, angle, velocity, particle, interpolation) in
      (&entities, &angels, &mut velocities, &mut particles, &mut interpolations).join()
    {
      particle.time_to_live -= time.as_secs_f32();
      if particle.time_to_live < 0.0 {
        entities.delete(e).unwrap();
        continue;
      }

      particle.x1 += velocity.x * f32::cos(angle.radians) * time.as_secs_f32();
      particle.y1 += velocity.y * f32::sin(angle.radians) * time.as_secs_f32();
      particle.x2 = particle.x1 + particle.length * f32::cos(angle.radians);
      particle.y2 = particle.y1 + particle.length * f32::sin(angle.radians);

      let (values, _) = interpolation.eval(time.as_secs_f32(), linear);
      velocity.x = values[0];
      velocity.y = values[0];
      particle.length = values[1];
      particle.width = values[2];
    }

    for event in events.read(
      self
        .reader_id
        .as_mut()
        .expect("reader_id Should not be None! Did you forget to initialize in setup()?"),
    ) {
      if let PlayerDeath(pos) = event {
        let rng = self
          .rng
          .as_mut()
          .expect("rng Should not be None! Did you forget to initialize in setup()?");
        for _ in 0..16 {
          let length = rng.gen_range(2.0..8.0);
          let time_to_live = rng.gen_range(0.3..0.5);
          let velocity = rng.gen_range(75.0..150.0);
          let radians = rng.gen_range(0.0..2.0 * PI);
          let width = 3.0;
          lazy
            .create_entity(&entities)
            .with(Angle { radians, velocity: 0.0 })
            .with(Velocity::new(velocity))
            .with(Interpolation::new(
              vec![(velocity, 0.0), (length, 0.0), (width, 1.0)], // can't tween the width to 0 because its not allowed by gfx thickline
              time_to_live,
            ))
            .with(LineParticle {
              width,
              color: Color::WHITE,
              length,
              x1: pos.x,
              y1: pos.y,
              x2: pos.x + length * f32::cos(radians),
              y2: pos.y + length * f32::sin(radians),
              time_to_live,
            })
            .build();
        }
      }
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.reader_id = Some(Write::<GameEventsChannel>::fetch(world).register_reader());
    self.rng = Some(rand::rngs::SmallRng::from_entropy());
  }

  fn dispose(self, _: &mut World)
  where
    Self: Sized,
  {
    drop(self.reader_id);
  }
}
