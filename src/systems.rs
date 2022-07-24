use crate::{
  components::{Angle, Geometry, Player, Velocity},
  environment::{SCREEN_HEIGHT, SCREEN_WIDTH},
  Camera, GameEvents, Position, Shake,
};
use bevy_ecs::prelude::*;
use sdl2::keyboard::Keycode;
use std::{collections::HashSet, time::Duration};

pub fn player_spawn_system(mut commands: Commands) {
  commands
    .spawn()
    .insert(Player)
    .insert(Position {
      x: SCREEN_WIDTH as f32 / 2.0,
      y: SCREEN_HEIGHT as f32 / 2.0,
    })
    .insert(Angle::default())
    .insert(Velocity::new(100.0))
    .insert(Geometry { buffers_idx: 0 });
}

pub fn player_system(
  mut query: Query<(&Player, &mut Position, &mut Angle, &mut Velocity)>,
  mut event_writer: EventWriter<GameEvents>,
  keycodes: Res<HashSet<Keycode>>,
  time: Res<Duration>,
) {
  for (_, mut position, mut angle, mut velocity) in query.iter_mut() {
    for keycode in keycodes.iter() {
      match keycode {
        Keycode::Left => angle.radians += angle.velocity * time.as_secs_f32(),
        Keycode::Right => angle.radians -= angle.velocity * time.as_secs_f32(),
        Keycode::S => {
          event_writer.send(GameEvents::PlayerDeath);
        }
        _ => {}
      }
    }
    println!("{position:?}");

    position.x += velocity.x * time.as_secs_f32() * f32::cos(angle.radians);
    position.y += velocity.y * time.as_secs_f32() * f32::sin(angle.radians);
    velocity.x = velocity.base_x;
    velocity.y = velocity.base_y;
  }
}

pub fn camera_shake_system(
  mut event_reader: EventReader<GameEvents>,
  mut camera: ResMut<Camera>,
  mut shake: ResMut<Shake>,
  time: Res<Duration>,
) {
  let Shake { is_shaking, .. } = *shake;

  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => shake.is_shaking = true,
    }
  }

  if is_shaking {
    shake.time += time.as_secs_f32();
    if shake.time > shake.duration {
      shake.time = 0.0;
      shake.is_shaking = false;
      return;
    }

    let s = shake.time * shake.frequency;
    let s0 = f32::floor(s);
    let s1 = s0 + 1.0;
    let k = if shake.time >= shake.duration {
      0.0
    } else {
      (shake.duration - shake.time) / shake.duration
    };

    fn noise(samples: &[f32]) -> impl Fn(f32) -> f32 + '_ {
      move |n| {
        let n = n as usize;
        if n >= samples.len() { 0.0 } else { samples[n] }
      }
    }
    let noise_x = noise(&shake.samples_x);
    let noise_y = noise(&shake.samples_y);
    let amplitude = |noise_fn: &dyn Fn(f32) -> f32| -> f32 {
      (noise_fn(s0) + (s - s0) * (noise_fn(s1) - noise_fn(s0))) * k * shake.amplitude
    };

    camera.camera_pos = glam::Vec3::new(amplitude(&noise_x), amplitude(&noise_y), 0.0);
  }
}
