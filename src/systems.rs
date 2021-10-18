use crate::{
  components::{Ammunition, AmmunitionRes, Angle, BoostRes, Player, Position, Velocity},
  environment::{SCREEN_HEIGHT, SCREEN_WIDTH},
  resources::{GameEvents, GameEventsChannel},
};
use ggez::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use specs::prelude::*;
use std::collections::HashSet;

pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, HashSet<winit::event::VirtualKeyCode>>,
    Read<'a, core::time::Duration>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, AmmunitionRes>,
    Write<'a, GameEventsChannel>,
    WriteStorage<'a, Velocity>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, Angle>,
    WriteStorage<'a, BoostRes>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, keycodes, dt, players, _, mut events, mut velocities, mut positions, mut angles, mut boosts) =
      data;

    for (_, e, velocity, position, angle, boost) in (
      &players,
      &entities,
      &mut velocities,
      &mut positions,
      &mut angles,
      &mut boosts,
    )
      .join()
    {
      for keycode in keycodes.iter() {
        match keycode {
          winit::event::VirtualKeyCode::Left => angle.radians -= angle.velocity * dt.as_secs_f32(),
          winit::event::VirtualKeyCode::Right => angle.radians += angle.velocity * dt.as_secs_f32(),
          winit::event::VirtualKeyCode::Up => {
            if boost.can_boost() {
              velocity.x *= 1.5;
              velocity.y *= 1.5;
              boost.boost -= boost.dec_amount * dt.as_secs_f32();
            }
          }
          winit::event::VirtualKeyCode::Down => {
            if boost.can_boost() {
              velocity.x *= 0.5;
              velocity.y *= 0.5;
              boost.boost -= boost.dec_amount * dt.as_secs_f32();
            }
          }
          winit::event::VirtualKeyCode::D => {
            entities.delete(e).unwrap();
            events.single_write(GameEvents::PlayerDeath(*position));
          }
          _ => {}
        }
      }

      if boost.is_empty() && boost.no_cooldown() {
        boost.cooldown = boost.cooldown_sec;
      } else if let Some(mut cooldown) = boost.cooldown.take() {
        cooldown -= dt.as_secs_f32();
        if cooldown > 0.0 {
          boost.cooldown.replace(cooldown);
        }
      }

      position.x += velocity.x * dt.as_secs_f32() * f32::cos(angle.radians);
      position.y += velocity.y * dt.as_secs_f32() * f32::sin(angle.radians);
      velocity.x = velocity.base_x;
      velocity.y = velocity.base_y;
      boost.boost = boost.max_boost.min(boost.boost + boost.inc_amount * dt.as_secs_f32());

      // let sprite_offset_x = sprite.width() / 2.0;
      // let sprite_offset_y = sprite.height() / 2.0;
      // if (position.x - sprite_offset_x) < 0.0
      //   || (position.x + sprite_offset_x) > SCREEN_WIDTH as f32
      //   || (position.y - sprite_offset_y) < 0.0
      //   || (position.y + sprite_offset_y) > SCREEN_HEIGHT as f32
      // {
      //   entities.delete(e).unwrap();
      //   events.single_write(GameEvents::PlayerDeath(*position));
      // }
    }

    // if keycodes.contains(&winit::event::VirtualKeyCode::S) {
    //   let number_of_players = (&players, &entities).join().count();
    //   if number_of_players == 0 {
    //     lazy
    //       .create_entity(&entities)
    //       .with(Player)
    //       .with(Position {
    //         x: SCREEN_WIDTH / 2.0,
    //         y: SCREEN_HEIGHT / 2.0,
    //       })
    //       .with(Angle::default())
    //       .with(Velocity::new(100.0))
    //       .with(BoostRes::default())
    //       .with(AmmunitionRes::default())
    //       .build();
    //     events.single_write(GameEvents::PlayerSpawn);
    //   }
    // }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    world
      .create_entity()
      .with(Player)
      .with(Position {
        x: SCREEN_WIDTH / 2.0,
        y: SCREEN_HEIGHT / 2.0 + 20.0,
      })
      .with(Angle::default())
      .with(Velocity::new(100.0))
      .with(BoostRes::default())
      .with(AmmunitionRes::default())
      .build();
  }
}

#[derive(Default)]
pub struct AmmunitionSystem {
  rng: Option<SmallRng>,
}

impl<'a> System<'a> for AmmunitionSystem {
  #[allow(clippy::type_complexity)]
  type SystemData = (
    Entities<'a>,
    Read<'a, LazyUpdate>,
    Read<'a, HashSet<winit::event::VirtualKeyCode>>,
    ReadStorage<'a, Ammunition>,
    WriteStorage<'a, Position>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (entities, lazy, keycodes, ammunition, mut positions) = data;

    if keycodes.contains(&winit::event::VirtualKeyCode::S) {
      let rng = self
        .rng
        .as_mut()
        .expect("rng Should not be None! Did you forget to initialize in setup()?");
      lazy
        .create_entity(&entities)
        .with(Ammunition)
        .with(Position {
          x: rng.gen_range(0.0..SCREEN_WIDTH),
          y: rng.gen_range(0.0..SCREEN_HEIGHT),
        })
        .build();
    }

    for (_, position) in (&ammunition, &mut positions).join() {
      position.x += 1.0;
      position.y += 1.0;
    }
  }

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.rng = Some(SmallRng::from_entropy());
  }
}
