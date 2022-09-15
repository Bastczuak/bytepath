use crate::{
  color::ColorGl,
  components::{Interpolation, Player, Projectile, Transform},
  easings::ease_in_out_cubic,
  environment::{RGB_COLOR_PLAYER, SCREEN_HEIGHT, SCREEN_WIDTH, Z_INDEX_PLAYER},
  render::WithTransformColor,
  resources::*,
  GameEvents,
};
use bevy_ecs::prelude::*;
use lyon::{
  geom::{Box2D, Size},
  lyon_tessellation::FillOptions,
  math::Point,
  tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator},
};
use sdl2::keyboard::Keycode;
use std::{collections::HashSet, time::Duration};

pub fn player_spawn_system(mut commands: Commands) {
  commands
      .spawn()
      .insert(Player {
        movement_speed: 100.0,
        rotation_speed: 360.0f32.to_radians(),
      })
      .insert(Transform {
        translation: glam::Vec3::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0, Z_INDEX_PLAYER),
        ..Default::default()
      })
    .insert(Interpolation::new(vec![(8.0, 0.0)], 0.24));
}

pub fn shooting_system(
  mut query: Query<(&Player, &Transform, &mut Interpolation)>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<FillTessellator>,
  time: Res<Duration>,
) {
  for (_, transform, mut interpolation) in query.iter_mut() {
    let (values, _) = interpolation.eval(time.as_secs_f32(), ease_in_out_cubic);
    let mat4 =
        glam::Mat4::from_rotation_translation(
          transform.rotation * glam::Quat::from_rotation_z(45.0f32.to_radians()),
          transform.translation,
        ) * glam::Mat4::from_translation(glam::vec3(8.0 - values[0] / 2.0, 8.0 - values[0] / 2.0, Z_INDEX_PLAYER));

    tessellator
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(values[0], values[0])),
          &FillOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(RGB_COLOR_PLAYER),
          },
        ),
      )
      .unwrap();
  }
}

pub fn player_system(
  mut query: Query<(&Player, &mut Transform)>,
  mut event_writer: EventWriter<GameEvents>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<StrokeTessellator>,
  keycodes: Res<HashSet<Keycode>>,
  time: Res<Duration>,
) {
  for (player, mut transform) in query.iter_mut() {
    let mut rotation_factor = 0.0;
    let mut movement_factor = 1.0;
    let time = time.as_secs_f32();

    for keycode in keycodes.iter() {
      match keycode {
        Keycode::Up => movement_factor = 1.5,
        Keycode::Left => rotation_factor += 1.0,
        Keycode::Right => rotation_factor -= 1.0,
        Keycode::S => {
          event_writer.send(GameEvents::PlayerDeath);
        }
        _ => {}
      }
    }

    transform.rotation *= glam::Quat::from_rotation_z(rotation_factor * player.rotation_speed * time);
    let movement_direction = transform.rotation * glam::Vec3::Y;
    let movement_distance = movement_factor * player.movement_speed * time;
    let translation_delta = movement_direction * movement_distance;
    // transform.translation += translation_delta;

    let mut options = StrokeOptions::default();
    options.line_width = 1.5;
    tessellator
      .tessellate_circle(
        Point::new(0.0, 0.0),
        12.0,
        &options,
        &mut BuffersBuilder::new(
          &mut circles.vertex_buffer,
          WithTransformColor {
            transform: transform.mat4(),
            color_rgba: ColorGl::from(RGB_COLOR_PLAYER),
          },
        ),
      )
      .unwrap();
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
        if n >= samples.len() {
          0.0
        } else {
          samples[n]
        }
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

pub fn projectile_spawn_system(
  query: Query<(&Player, &Transform)>,
  mut commands: Commands,
  time: Res<Duration>,
  mut config: ResMut<ProjectileSpawnConfig>,
  keycodes: Res<HashSet<Keycode>>,
) {
  for (player, transform) in query.iter() {
    config.timer += *time;

    if config.timer.as_secs_f32() >= 0.25 {
      config.timer = Duration::default();

      let movement_direction = transform.rotation * glam::Vec3::Y;
      let translation_delta = movement_direction * 12.0;
      let translation = transform.translation + translation_delta;

      commands
          .spawn()
          .insert(Transform {
            translation,
            ..*transform
          })
          .insert(Projectile {
            movement_speed: player.movement_speed,
          });

      if keycodes.contains(&Keycode::Space) {
        let movement_direction = transform.rotation * glam::vec3(1.0, 1.0, 0.0);
        let translation_delta = movement_direction * 12.0;
        let translation = transform.translation + translation_delta;

        commands
            .spawn()
            .insert(Transform {
              translation,
              ..*transform
            })
            .insert(Projectile {
              movement_speed: player.movement_speed,
            });

        let movement_direction = transform.rotation * glam::vec3(-1.0, 1.0, 0.0);
        let translation_delta = movement_direction * 12.0;
        let translation = transform.translation + translation_delta;

        commands
            .spawn()
            .insert(Transform {
              translation,
              ..*transform
            })
            .insert(Projectile {
              movement_speed: player.movement_speed,
            });
      }
    }
  }
}

pub fn projectile_system(
  mut query: Query<(&Projectile, &mut Transform)>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<StrokeTessellator>,
  time: Res<Duration>,
) {
  for (projectile, mut transform) in query.iter_mut() {
    let time = time.as_secs_f32();
    let movement_direction = transform.rotation * glam::Vec3::Y;
    let movement_distance = projectile.movement_speed * time;
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta;

    let mut options = StrokeOptions::default();
    options.line_width = 1.5;
    tessellator
        .tessellate_circle(
          Point::new(0.0, 0.0),
          2.5,
          &options,
          &mut BuffersBuilder::new(
            &mut circles.vertex_buffer,
            WithTransformColor {
              transform: transform.mat4(),
              color_rgba: ColorGl::from(RGB_COLOR_PLAYER),
            },
          ),
        )
        .unwrap();
  }
}
