use crate::{Camera, Shake};
use bevy_ecs::prelude::*;
use sdl2::keyboard::Keycode;
use std::{collections::HashSet, time::Duration};

pub fn camera_shake(
  mut camera: ResMut<Camera>,
  mut shake: ResMut<Shake>,
  keycodes: Res<HashSet<Keycode>>,
  time: Res<Duration>,
) {
  for keycode in keycodes.iter() {
    if keycode == &Keycode::S {
      shake.is_shaking = true;
    }
  }
  let Shake { is_shaking, .. } = *shake;
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
