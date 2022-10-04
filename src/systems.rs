use crate::{
  color::ColorGl, components::*, easings::*, environment::*, render::WithTransformColor, resources::*, GameEvents,
};
use bevy_ecs::prelude::*;
use glam::Vec3Swizzles;
use lyon::{
  geom::{Box2D, Size},
  lyon_tessellation::FillOptions,
  math::{point, Point},
  path::Path,
  tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator},
};
use rand::Rng;
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
  time: Res<Time>,
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
  mut commands: Commands,
  mut query: Query<(&Player, &mut Transform, Entity)>,
  mut event_writer: EventWriter<GameEvents>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<StrokeTessellator>,
  keycodes: Res<HashSet<Keycode>>,
  time: Res<Time>,
) {
  for (player, mut transform, entity) in query.iter_mut() {
    let mut rotation_factor = 0.0;
    let mut movement_factor = 1.0;
    let time = time.as_secs_f32();

    for keycode in keycodes.iter() {
      match keycode {
        Keycode::Up => movement_factor = 1.5,
        Keycode::Left => rotation_factor += 1.0,
        Keycode::Right => rotation_factor -= 1.0,
        Keycode::Down => movement_factor = 0.5,
        Keycode::S => {
          event_writer.send(GameEvents::PlayerDeath);
          commands.entity(entity).despawn();
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

pub fn player_explosion_spawn_system(
  mut commands: Commands,
  mut event_reader: EventReader<GameEvents>,
  query: Query<(&Player, &Transform)>,
  mut rng: ResMut<rand::rngs::SmallRng>,
) {
  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => {
        for (_, transform) in query.iter() {
          for _ in 0..rng.gen_range(8usize..12usize) {
            let length = rng.gen_range(2.0..8.0);
            let width = 3.0;
            let time_to_live = rng.gen_range(0.3..0.5);
            let movement_speed = rng.gen_range(75.0..150.0);
            let z_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);

            commands
              .spawn()
              .insert(Transform {
                rotation: glam::Quat::from_rotation_z(z_angle),
                ..*transform
              })
              .insert(PlayerExplosion {
                timer: Duration::default(),
                time_to_live,
              })
              .insert(Interpolation::new(
                vec![(movement_speed, 0.0), (length, 0.0), (width, 0.0)],
                time_to_live,
              ));
          }
        }
      }
    }
  }
}

pub fn player_explosion_system(
  mut commands: Commands,
  mut query: Query<(&mut PlayerExplosion, &mut Transform, &mut Interpolation, Entity)>,
  mut lines: ResMut<LineGeometry>,
  mut tessellator: ResMut<StrokeTessellator>,
  time: Res<Time>,
) {
  for (mut explosion, mut transform, mut interpolation, entity) in query.iter_mut() {
    explosion.timer += **time;
    if explosion.timer.as_secs_f32() >= explosion.time_to_live {
      commands.entity(entity).despawn();
      continue;
    }

    let (values, _) = interpolation.eval(time.as_secs_f32(), linear);
    let movement_speed = values[0];
    let length = values[1];
    let width = values[2];
    let movement_direction = transform.rotation * glam::Vec3::Y;
    let movement_distance = movement_speed * time.as_secs_f32();
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta;

    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(0.0, length));
    builder.close();

    let mut options = StrokeOptions::default();
    options.line_width = width;
    tessellator
      .tessellate_path(
        &builder.build(),
        &options,
        &mut BuffersBuilder::new(
          &mut lines.vertex_buffer,
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
  raw_time: Res<Duration>, // don't use Res<Time> here because I don't want to apply slow motion to camera shake
) {
  let Shake { is_shaking, .. } = *shake;

  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => shake.is_shaking = true,
    }
  }

  if is_shaking {
    shake.time += raw_time.as_secs_f32();
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

pub fn screen_flash_system(
  mut event_reader: EventReader<GameEvents>,
  mut flash: ResMut<Flash>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<FillTessellator>,
) {
  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => flash.is_flashing = true,
    }
  }

  if flash.is_flashing {
    flash.frame_cnt -= 1;

    if flash.frame_cnt > 0 {
      tessellator
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32)),
          &FillOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 100.0)),
              color_rgba: ColorGl::from(RGB_COLOR_PLAYER),
            },
          ),
        )
        .unwrap();
    } else {
      *flash = Flash::default();
    }
  }
}

pub fn projectile_spawn_system(
  query: Query<(&Player, &Transform)>,
  mut commands: Commands,
  time: Res<Time>,
  mut timer: ResMut<EntitySpawnTimer>,
  keycodes: Res<HashSet<Keycode>>,
) {
  for (player, transform) in query.iter() {
    timer.projectile += **time;

    if timer.projectile.as_secs_f32() >= 0.25 {
      timer.projectile = Duration::default();

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
  time: Res<Time>,
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

    tessellator
      .tessellate_circle(
        Point::new(0.0, 0.0),
        2.5,
        &StrokeOptions::default(),
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
  time: Res<Time>,
) {
  for (mut dead_projectile, transform, entity) in query.iter_mut() {
    dead_projectile.timer += **time;

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

pub fn timing_system(
  mut event_reader: EventReader<GameEvents>,
  raw_time: Res<Duration>, // this is set in main() with *world.resource_mut() = dt;
  mut time: ResMut<Time>,
) {
  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => time.slow_down_timer = Some(Duration::default()),
    }
  }

  if let Some(mut timer) = time.slow_down_timer.take() {
    timer += *raw_time;
    if timer.as_secs_f32() <= SLOW_DOWN_DURATION_ON_DEATH {
      let easing = ease_in_out_cubic(timer.as_secs_f32() / SLOW_DOWN_DURATION_ON_DEATH);
      let slow_amount = (1.0 - easing) * 0.15 + easing * 1.0;
      **time = Duration::from_secs_f32(raw_time.as_secs_f32() * slow_amount);
      time.slow_down_timer.replace(timer);
    }
  } else {
    **time = *raw_time;
  }
}

pub fn tick_effect_spawn_system(
  query: Query<&Player>,
  mut commands: Commands,
  time: Res<Time>,
  mut timer: ResMut<EntitySpawnTimer>,
) {
  for _ in query.iter() {
    timer.tick_effect += **time;

    if timer.tick_effect.as_secs_f32() >= 5.0 {
      timer.tick_effect = Duration::default();

      commands
        .spawn()
        .insert(TickEffect {
          timer: Duration::default(),
        })
        .insert(Interpolation::new(vec![(32.0, 0.0)], 0.13));
    }
  }
}

pub fn tick_effect_system(
  mut commands: Commands,
  player_query: Query<(&Player, &Transform)>,
  mut tick_effect_query: Query<(&mut TickEffect, &mut Interpolation, Entity)>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<FillTessellator>,
  time: Res<Time>,
) {
  for (_, transform) in player_query.iter() {
    for (mut tick_effect, mut interpolation, entity) in tick_effect_query.iter_mut() {
      tick_effect.timer += **time;

      if tick_effect.timer.as_secs_f32() <= 0.13 {
        let (values, _) = interpolation.eval(time.as_secs_f32(), ease_in_out_cubic);
        let mat4 = glam::Mat4::from_translation(transform.translation)
          * glam::Mat4::from_translation(glam::vec3(48.0 / -2.0, 32.0 / 2.0 - values[0], Z_INDEX_PLAYER));

        tessellator
          .tessellate_rectangle(
            &Box2D::from_size(Size::new(48.0, values[0])),
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
      } else {
        commands.entity(entity).despawn();
      }
    }
  }
}
