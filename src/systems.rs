use crate::{
  color::ColorGl,
  components::{DeadProjectile, Interpolation, Player, Projectile, Transform},
  easings::ease_in_out_cubic,
  environment::*,
  render::WithTransformColor,
  resources::*,
  GameEvents,
};
use bevy_ecs::prelude::*;
use glam::Vec3Swizzles;
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
    transform.translation += translation_delta;

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
            movement_speed: player.movement_speed * 2.0,
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
              movement_speed: player.movement_speed * 2.0,
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
              movement_speed: player.movement_speed * 2.0,
            });
      }
    }
  }
}

pub fn projectile_system(
  mut commands: Commands,
  mut query: Query<(&Projectile, &mut Transform, Entity)>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<StrokeTessellator>,
  time: Res<Duration>,
) {
  for (projectile, mut transform, entity) in query.iter_mut() {
    let pos = transform.translation.xy();
    if pos.x < 0.0 || pos.x > SCREEN_WIDTH as f32 || pos.y < 0.0 || pos.y > SCREEN_HEIGHT as f32 {
      commands.entity(entity).despawn();

      let clamped_x = pos.x.max(0.0).min(SCREEN_WIDTH as f32 - DEAD_PROJECTILE_HEIGHT);
      let clamped_y = pos.y.max(0.0).min(SCREEN_HEIGHT as f32 - DEAD_PROJECTILE_HEIGHT);
      let translation = glam::vec3(clamped_x, clamped_y, 1.0);
      let rotation = if pos.x < 0.0 || pos.x > SCREEN_WIDTH as f32 {
        glam::Quat::from_rotation_z(-std::f32::consts::PI / 2.0)
      } else {
        glam::Quat::from_rotation_z(0.0)
      };

      commands
          .spawn()
          .insert(Transform { translation, rotation })
          .insert(DeadProjectile {
            timer: Duration::default(),
          });
    }

    let movement_direction = transform.rotation * glam::Vec3::Y;
    let movement_distance = projectile.movement_speed * time.as_secs_f32();
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

pub fn projectile_death_system(
  mut commands: Commands,
  mut query: Query<(&mut DeadProjectile, &Transform, Entity)>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<FillTessellator>,
  time: Res<Duration>,
) {
  for (mut dead_projectile, transform, entity) in query.iter_mut() {
    dead_projectile.timer += *time;

    if dead_projectile.timer.as_secs_f32() >= 0.25 {
      commands.entity(entity).despawn();
      continue;
    }

    let color_rgba = if dead_projectile.timer.as_secs_f32() >= 0.1 {
      ColorGl::from(RGB_COLOR_DEATH)
    } else {
      ColorGl::from(RGB_COLOR_PLAYER)
    };
    let transform = glam::Mat4::from_rotation_translation(transform.rotation, transform.translation);
    tessellator
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(DEAD_PROJECTILE_WIDTH, DEAD_PROJECTILE_HEIGHT)),
          &FillOptions::default(),
          &mut BuffersBuilder::new(&mut quads.vertex_buffer, WithTransformColor { transform, color_rgba }),
        )
        .unwrap();
  }
}
