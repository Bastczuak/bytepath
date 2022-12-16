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
  tessellation::{BuffersBuilder, StrokeOptions},
};
use rand::Rng;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn screen_ouf_of_bounds_test(position: glam::Vec2, offset: Option<f32>) -> bool {
  let offset = offset.unwrap_or_default();
  position.x < -offset
    || position.x > SCREEN_WIDTH as f32 + offset
    || position.y < -offset
    || position.y > SCREEN_HEIGHT as f32 + offset
}

pub fn player_spawn_system(mut commands: Commands) {
  commands
      .spawn_empty()
      .insert(Player {
        movement_speed: 100.0,
        rotation_speed: 360.0f32.to_radians(),
      })
      .insert(Transform {
        translation: glam::Vec3::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0, Z_INDEX_PLAYER),
        ..Default::default()
      })
      .insert(Boost::default())
      .insert(Interpolation::new(vec![(8.0, 0.0)], 0.24, true));
}

pub fn shooting_system(
  mut query: Query<(&Player, &Transform, &mut Interpolation)>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<Fills>,
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
  mut query: Query<(&Player, &mut Transform, &mut Boost, Entity)>,
  mut event_writer: EventWriter<GameEvents>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<Strokes>,
  keycodes: Res<KeyCodes>,
  time: Res<Time>,
) {
  for (player, mut transform, mut boost, entity) in query.iter_mut() {
    let mut rotation_factor = 0.0;
    let mut movement_factor = 1.0;
    let time = time.as_secs_f32();

    for keycode in keycodes.iter() {
      match keycode {
        Keycode::Up => {
          if boost.can_boost() {
            movement_factor = 1.5;
            boost.boost -= boost.dec_amount * time;
          }
        }
        Keycode::Down => {
          if boost.can_boost() {
            movement_factor = 0.5;
            boost.boost -= boost.dec_amount * time;
          }
        }
        Keycode::Left => rotation_factor += 1.0,
        Keycode::Right => rotation_factor -= 1.0,
        Keycode::S => {
          event_writer.send(GameEvents::PlayerDeath);
          commands.entity(entity).despawn();
        }
        _ => {}
      }
    }

    if boost.is_empty() && boost.no_cooldown() {
      boost.cooldown = boost.cooldown_sec;
    } else if let Some(mut cooldown) = boost.cooldown.take() {
      cooldown -= time;
      if cooldown > 0.0 {
        boost.cooldown.replace(cooldown);
      }
    }
    boost.boost = boost.max_boost.min(boost.boost + boost.inc_amount * time);

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

pub fn trail_effect_spawn_system(
  mut commands: Commands,
  query: Query<(&Player, &Transform)>,
  mut rng: ResMut<Randoms>,
) {
  for (_, transform) in query.iter() {
    let radius = rng.gen_range(4.0..6.0);
    let movement_direction = transform.rotation * glam::Vec3::Y;
    let translation_delta = movement_direction * (12.0 + 2.0);
    let translation = transform.translation - translation_delta + glam::vec3(0.0, 0.0, Z_INDEX_TRAIL_EFFECT);
    let time_to_live = rng.gen_range(0.15..0.25);

    commands
        .spawn_empty()
        .insert(TrailEffect)
        .insert(Interpolation::new(vec![(radius, 0.0)], time_to_live, true))
      .insert(Transform {
        translation,
        ..*transform
      });
  }
}

pub fn trail_effect_system(
  mut commands: Commands,
  mut query: Query<(&TrailEffect, &mut Interpolation, &Transform, Entity)>,
  boost: Query<&Boost>,
  mut circles: ResMut<CircleGeometry>,
  mut tessellator: ResMut<Fills>,
  keycodes: Res<KeyCodes>,
  time: Res<Time>,
) {
  for (_, mut interpolation, transform, entity) in query.iter_mut() {
    let (values, done) = interpolation.eval(time.as_secs_f32(), linear);
    if done {
      commands.entity(entity).despawn();
      continue;
    }

    let mut color_rgba = ColorGl::from(RGB_COLOR_TRAIL);

    if let Ok(boost) = boost.get_single() {
      if boost.can_boost() {
        for keycode in keycodes.iter() {
          match keycode {
            Keycode::Up => color_rgba = ColorGl::from(RGB_COLOR_BOOST),
            Keycode::Down => color_rgba = ColorGl::from(RGB_COLOR_BOOST),
            _ => {}
          }
        }
      }
    }

    tessellator
      .tessellate_circle(
        Point::new(0.0, 0.0),
        values[0],
        &FillOptions::default(),
        &mut BuffersBuilder::new(
          &mut circles.vertex_buffer,
          WithTransformColor {
            transform: transform.mat4(),
            color_rgba,
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
  mut rng: ResMut<Randoms>,
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
                .spawn_empty()
              .insert(Transform {
                rotation: glam::Quat::from_rotation_z(z_angle),
                ..*transform
              })
              .insert(ExplosionEffect {
                color: ColorGl::from(RGB_COLOR_PLAYER),
              })
              .insert(Interpolation::new(
                vec![(movement_speed, 0.0), (length, 0.0), (width, 0.0)],
                time_to_live,
                false,
              ));
          }
        }
      }
    }
  }
}

pub fn explosion_system(
  mut commands: Commands,
  mut query: Query<(&ExplosionEffect, &mut Transform, &mut Interpolation, Entity)>,
  mut lines: ResMut<LineGeometry>,
  mut tessellator: ResMut<Strokes>,
  time: Res<Time>,
) {
  for (explosion, mut transform, mut interpolation, entity) in query.iter_mut() {
    let (values, done) = interpolation.eval(time.as_secs_f32(), linear);
    if done {
      commands.entity(entity).despawn();
      continue;
    }

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
            color_rgba: explosion.color,
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
  raw_time: Res<DurationWrapper>, // don't use Res<Time> here because I don't want to apply slow motion to camera shake
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

pub fn screen_flash_system(
  mut event_reader: EventReader<GameEvents>,
  mut flash: ResMut<Flash>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<Fills>,
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
  timer: Res<EntitySpawnTimer>,
  keycodes: Res<KeyCodes>,
) {
  for (player, transform) in query.iter() {
    if timer.projectile.finished {
      let movement_direction = transform.rotation * glam::Vec3::Y;
      let translation_delta = movement_direction * 12.0;
      let translation = transform.translation + translation_delta;

      commands
          .spawn_empty()
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
            .spawn_empty()
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
            .spawn_empty()
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
  mut tessellator: ResMut<Strokes>,
  time: Res<Time>,
) {
  for (projectile, mut transform, entity) in query.iter_mut() {
    let pos = transform.translation.xy();
    if screen_ouf_of_bounds_test(pos, None) {
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
          .spawn_empty()
        .insert(Transform {
          translation,
          rotation,
          ..Default::default()
        })
        .insert(DeadProjectile {
          timer: Timer::from_seconds(0.25, false),
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
  mut tessellator: ResMut<Fills>,
  time: Res<Time>,
) {
  for (mut dead_projectile, transform, entity) in query.iter_mut() {
    dead_projectile.timer.tick(**time);

    if dead_projectile.timer.finished {
      commands.entity(entity).despawn();
      continue;
    }

    let color_rgba = if dead_projectile.timer.elapsed >= 0.1 {
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
  mut timers: ResMut<EntitySpawnTimer>,
  raw_time: Res<DurationWrapper>, // this is set in main() with *world.resource_mut() = dt;
  mut time: ResMut<Time>,
) {
  for event in event_reader.iter() {
    match event {
      GameEvents::PlayerDeath => time.slow_down_timer = Some(Duration::default()),
    }
  }

  if let Some(mut timer) = time.slow_down_timer.take() {
    timer += **raw_time;
    if timer.as_secs_f32() <= SLOW_DOWN_DURATION_ON_DEATH {
      let easing = ease_in_out_cubic(timer.as_secs_f32() / SLOW_DOWN_DURATION_ON_DEATH);
      let slow_amount = (1.0 - easing) * 0.15 + easing * 1.0;
      **time = Duration::from_secs_f32(raw_time.as_secs_f32() * slow_amount);
      time.slow_down_timer.replace(timer);
    }
  } else {
    **time = **raw_time;
  }

  for timer in timers.as_array() {
    timer.tick(**time);
  }
}

pub fn tick_effect_spawn_system(query: Query<&Player>, mut commands: Commands, timer: Res<EntitySpawnTimer>) {
  for _ in query.iter() {
    if timer.tick_effect.finished {
      commands
          .spawn_empty()
          .insert(TickEffect)
          .insert(Interpolation::new(vec![(32.0, 0.0)], 0.13, true));
    }
  }
}

pub fn tick_effect_system(
  mut commands: Commands,
  player_query: Query<(&Player, &Transform)>,
  mut tick_effect_query: Query<(&TickEffect, &mut Interpolation, Entity)>,
  mut quads: ResMut<QuadGeometry>,
  mut tessellator: ResMut<Fills>,
  time: Res<Time>,
) {
  for (_, transform) in player_query.iter() {
    for (_, mut interpolation, entity) in tick_effect_query.iter_mut() {
      let (values, done) = interpolation.eval(time.as_secs_f32(), ease_in_out_cubic);
      if done {
        commands.entity(entity).despawn();
        continue;
      }

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
    }
  }
}

pub fn ammo_pickup_spawn_system(mut commands: Commands, timer: Res<EntitySpawnTimer>, mut rng: ResMut<Randoms>) {
  if timer.ammo_pickup.finished {
    let x = rng.gen_range(8.0..SCREEN_WIDTH as f32 - 8.0);
    let y = rng.gen_range(8.0..SCREEN_HEIGHT as f32 - 8.0);
    let rotation = glam::Quat::from_rotation_z(rng.gen_range(0.0..2.0 * std::f32::consts::PI));
    let movement_speed = rng.gen_range(10.0..20.0);
    let rotation_speed = std::f32::consts::PI;

    commands
        .spawn_empty()
        .insert(AmmoPickup {
          movement_speed,
          rotation_speed,
          center_rotation_speed: rng.gen_range(-2.0 * std::f32::consts::PI..2.0 * std::f32::consts::PI),
          timer: Timer::from_seconds(0.15, false),
        })
      .insert(Transform {
        translation: glam::vec3(x, y, Z_INDEX_AMMO_PICKUP),
        rotation,
        ..Default::default()
      });
  }
}

pub fn ammo_pickup_system(
  mut commands: Commands,
  player_query: Query<&Transform, With<Player>>,
  mut query: Query<(&mut AmmoPickup, &mut Transform, Entity), Without<Player>>,
  mut quads: ResMut<QuadGeometry>,
  mut strokes: ResMut<Strokes>,
  mut fills: ResMut<Fills>,
  time: Res<Time>,
  mut rng: ResMut<Randoms>,
) {
  for (mut ammo, mut transform, entity) in query.iter_mut() {
    let pos = transform.translation.xy();
    if screen_ouf_of_bounds_test(pos, Some(8.0)) {
      commands.entity(entity).despawn();
      continue;
    }

    if ammo.timer.finished {
      commands.entity(entity).despawn();
      continue;
    }

    if ammo.timer.elapsed > 0.0 {
      ammo.timer.tick(**time);
      let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(9.5 / -2.0, 9.5 / -2.0, 1.0));

      fills
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(9.5, 9.5)),
          &FillOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(RGB_COLOR_AMMO_PICKUP),
            },
          ),
        )
        .unwrap();
      continue;
    }

    if let Ok(player) = player_query.get_single() {
      let player_translation = player.translation.xy();
      let ammo_forward = (transform.rotation * glam::Vec3::Y).xy();
      let to_player = (player_translation - transform.translation.xy()).normalize();
      let forward_dot_player = ammo_forward.dot(to_player);

      if (forward_dot_player - 1.0).abs() < f32::EPSILON {
        continue;
      }

      let ammo_right = (transform.rotation * glam::Vec3::X).xy();
      let right_to_player = ammo_right.dot(to_player);
      let rotation_sign = -f32::copysign(1.0, right_to_player);
      let max_angle = forward_dot_player.clamp(-1.0, 1.0).acos();
      let rotation_angle = rotation_sign * (ammo.rotation_speed * time.as_secs_f32()).min(max_angle);
      transform.rotation *= glam::Quat::from_rotation_z(rotation_angle);

      let distance = (transform.translation - player.translation).length();
      if distance < 8.0 + 12.0 {
        ammo.timer.tick(**time);

        for _ in 0..rng.gen_range(4usize..8usize) {
          let length = 5.0;
          let width = 3.0;
          let time_to_live = rng.gen_range(0.2..0.4);
          let movement_speed = rng.gen_range(75.0..150.0);
          let z_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);

          commands
              .spawn_empty()
            .insert(Transform {
              rotation: glam::Quat::from_rotation_z(z_angle),
              ..*transform
            })
            .insert(ExplosionEffect {
              color: ColorGl::from(RGB_COLOR_AMMO_PICKUP),
            })
            .insert(Interpolation::new(
              vec![(movement_speed, 0.0), (length, 0.0), (width, 0.0)],
              time_to_live,
              false,
            ));
        }
      }
    }

    transform.center_rotation *= glam::Quat::from_rotation_z(ammo.center_rotation_speed * time.as_secs_f32());
    let movement_direction = transform.rotation * glam::Vec3::Y;
    let movement_distance = ammo.movement_speed * time.as_secs_f32();
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta;

    let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(8.0 / -2.0, 8.0 / -2.0, 1.0));

    strokes
      .tessellate_rectangle(
        &Box2D::from_size(Size::new(8.0, 8.0)),
        &StrokeOptions::default(),
        &mut BuffersBuilder::new(
          &mut quads.vertex_buffer,
          WithTransformColor {
            transform: mat4,
            color_rgba: ColorGl::from(RGB_COLOR_AMMO_PICKUP),
          },
        ),
      )
        .unwrap();
  }
}

pub fn boost_pickup_spawn_system(mut commands: Commands, timer: Res<EntitySpawnTimer>, mut rng: ResMut<Randoms>) {
  if timer.boost_pickup.finished {
    let movement_direction = if rng.gen_bool(1.0 / 2.0) { -1.0 } else { 1.0 };
    let x = if movement_direction > 0.0 {
      -12.0
    } else {
      SCREEN_WIDTH as f32 + 12.0
    };
    let y = rng.gen_range(12.0..SCREEN_HEIGHT as f32 - 12.0);
    let movement_speed = rng.gen_range(20.0..40.0);

    commands
        .spawn_empty()
        .insert(BoostPickup {
          movement_direction,
          movement_speed,
          center_rotation_speed: rng.gen_range(-2.0 * std::f32::consts::PI..2.0 * std::f32::consts::PI),
          visible: true,
          timer: Timer::from_seconds(0.05, true),
        })
        .insert(Transform {
          translation: glam::vec3(x, y, Z_INDEX_BOOST_PICKUP),
          ..Default::default()
        });
  }
}

pub fn boost_pickup_system(
  mut commands: Commands,
  player_query: Query<&Transform, With<Player>>,
  mut set: ParamSet<(
    Query<(&BoostPickup, &mut Transform, Entity), (Without<Player>, Without<Interpolation>)>,
    Query<(&mut BoostPickup, &Transform, &mut Interpolation, Entity), Without<Player>>,
  )>,
  mut quads: ResMut<QuadGeometry>,
  mut strokes: ResMut<Strokes>,
  mut fills: ResMut<Fills>,
  time: Res<Time>,
) {
  for (mut boost, transform, mut interpolation, entity) in set.p1().iter_mut() {
    boost.timer.tick(**time);
    let time = time.as_secs_f32();
    let (values, done) = interpolation.eval(time, ease_in_out_cubic);
    let color = if boost.timer.count > 3 {
      RGB_COLOR_BOOST
    } else {
      RGB_COLOR_PLAYER
    };

    if boost.timer.count == 10 {
      commands.entity(entity).despawn();
      continue;
    }

    if boost.timer.finished && boost.timer.count > 3 {
      boost.visible = !boost.visible;
    }

    if !boost.visible {
      continue;
    }

    let size = 12.0;
    let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(size / -2.0, size / -2.0, 1.0));
    fills
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(size, size)),
          &FillOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(color),
            },
          ),
        )
        .unwrap();

    let size = if done { 12.0 * 1.5 * 2.0 } else { 12.0 * 1.5 * values[0] };
    let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(size / -2.0, size / -2.0, 1.0));
    strokes
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(size, size)),
          &StrokeOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(color),
            },
          ),
        )
        .unwrap();
  }

  let time = time.as_secs_f32();

  for (boost, mut transform, entity) in set.p0().iter_mut() {
    if screen_ouf_of_bounds_test(transform.translation.xy(), Some(12.0 * 1.5)) {
      commands.entity(entity).despawn();
      continue;
    }

    if let Ok(player) = player_query.get_single() {
      let distance = (transform.translation - player.translation).length();
      if distance < 12.0 * 0.5 + 12.0 {
        commands
            .entity(entity)
            .insert(Interpolation::new(vec![(1.0, 2.0)], 0.3, false));
        continue;
      }
    }

    transform.center_rotation *= glam::Quat::from_rotation_z(boost.center_rotation_speed * time);
    let movement_direction = glam::Vec3::X * boost.movement_direction;
    let movement_distance = boost.movement_speed * time;
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta;

    let size = 12.0 * 0.5;
    let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(size / -2.0, size / -2.0, 1.0));
    fills
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(size, size)),
          &FillOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(RGB_COLOR_BOOST),
            },
          ),
        )
        .unwrap();

    let size = 12.0 * 1.5;
    let mat4 = transform.mat4_center() * glam::Mat4::from_translation(glam::vec3(size / -2.0, size / -2.0, 1.0));
    strokes
        .tessellate_rectangle(
          &Box2D::from_size(Size::new(size, size)),
          &StrokeOptions::default(),
          &mut BuffersBuilder::new(
            &mut quads.vertex_buffer,
            WithTransformColor {
              transform: mat4,
              color_rgba: ColorGl::from(RGB_COLOR_BOOST),
            },
          ),
        )
        .unwrap();
  }
}
