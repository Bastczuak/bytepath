// fn create_ship_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 32, 32)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.circle(16, 16, 15, Color::WHITE).unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_shooting_effect_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 8, 8)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.box_(0, 0, 8, 8, Color::WHITE).unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_projectile_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 8, 8)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.circle(4, 4, 3, Color::WHITE).unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_projectile_death_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 6, 6)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.set_draw_color(Color::WHITE);
//       texture_canvas.box_(0, 0, 6, 3, Color::WHITE).unwrap();
//       texture_canvas.box_(0, 3, 6, 6, Color::from(RGB_COLOR_DEATH)).unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_tick_effect_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 48, 23)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.box_(0, 0, 48, 23, Color::WHITE).unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_trail_effect_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 96, 32)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas
//         .filled_circle(16, 16, 15, Color::from(RGB_COLOR_NON_BOOST))
//         .unwrap();
//       texture_canvas
//         .filled_circle(48, 16, 15, Color::from(RGB_COLOR_BOOST))
//         .unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_ammunition_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 18, 6)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas
//         .rectangle(0, 0, 6, 6, Color::from(RGB_COLOR_AMMUNITION))
//         .unwrap();
//       texture_canvas.box_(6, 0, 12, 6, Color::WHITE).unwrap();
//       texture_canvas
//         .box_(12, 0, 18, 6, Color::from(RGB_COLOR_AMMUNITION))
//         .unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn create_boost_texture<'a, 'b>(
//   texture_creator: &'a TextureCreator<WindowContext>,
//   canvas: &'b mut WindowCanvas,
// ) -> Result<Texture<'a>, String> {
//   let mut texture = texture_creator
//     .create_texture_target(texture_creator.default_pixel_format(), 18, 18)
//     .map_err(|e| e.to_string())?;
//   canvas
//     .with_texture_canvas(&mut texture, |texture_canvas| {
//       texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
//       texture_canvas.clear();
//       texture_canvas.box_(6, 6, 11, 11, Color::from(RGB_COLOR_BOOST)).unwrap();
//       texture_canvas
//         .rectangle(0, 0, 18, 18, Color::from(RGB_COLOR_BOOST))
//         .unwrap();
//     })
//     .map_err(|e| e.to_string())?;
//   texture.set_blend_mode(BlendMode::Blend);
//
//   Ok(texture)
// }
//
// fn main2() -> Result<(), String> {
//   let sdl_context = sdl2::init()?;
//   let sdl_video = sdl_context.video()?;
//   let display_mode = sdl_video.desktop_display_mode(0)?;
//   let factor = std::cmp::min(
//     display_mode.w / SCREEN_WIDTH as i32,
//     display_mode.h / SCREEN_HEIGHT as i32,
//   ) as u32;
//   let sdl_window = sdl_video
//     .window("bytepath", SCREEN_WIDTH * factor, SCREEN_HEIGHT * factor)
//     .position_centered()
//     .build()
//     .map_err(|e| e.to_string())?;
//
//   let mut canvas = sdl_window
//     .into_canvas()
//     .accelerated()
//     .present_vsync()
//     .build()
//     .map_err(|e| e.to_string())?;
//   canvas
//     .set_logical_size(SCREEN_WIDTH, SCREEN_HEIGHT)
//     .map_err(|e| e.to_string())?;
//   canvas.set_draw_color(Color::from(RGB_COLOR_BACKGROUND));
//   canvas.clear();
//   canvas.present();
//
//   let texture_creator = canvas.texture_creator();
//   let textures = [
//     create_ship_texture(&texture_creator, &mut canvas)?,
//     create_shooting_effect_texture(&texture_creator, &mut canvas)?,
//     create_projectile_texture(&texture_creator, &mut canvas)?,
//     create_projectile_death_texture(&texture_creator, &mut canvas)?,
//     create_tick_effect_texture(&texture_creator, &mut canvas)?,
//     create_trail_effect_texture(&texture_creator, &mut canvas)?,
//     create_ammunition_texture(&texture_creator, &mut canvas)?,
//     create_boost_texture(&texture_creator, &mut canvas)?,
//   ];
//
//   let mut dispatcher = DispatcherBuilder::new()
//     .with(ShakeSystem::default(), "shake_system", &[])
//     .with(FlashSystem::default(), "flash_system", &[])
//     .with(LineParticleSystem::default(), "line_particle_system", &[])
//     .with(PlayerSystem, "player_system", &[])
//     .with(ShootingSystem::default(), "shooting_system", &["player_system"])
//     .with(ProjectileSystem::default(), "projectile_system", &["player_system"])
//     .with(TickEffectSystem::default(), "tick_effect_system", &["player_system"])
//     .with(TrailEffectSystem::default(), "trail_effect_system", &["player_system"])
//     .with(AmmunitionSystem::default(), "ammunition_system", &["player_system"])
//     .with(BoostSystem::default(), "boost_system", &["player_system"])
//     .with(
//       AmmunitionDeathSystem,
//       "ammunition_death_system",
//       &["player_system", "ammunition_system"],
//     )
//     .with(
//       ProjectileDeathSystem::default(),
//       "projectile_death_system",
//       &["projectile_system"],
//     )
//     .build();
//   let mut world = World::new();
//   dispatcher.setup(&mut world);
//   render::RenderSystemData::setup(&mut world);
//
//   let mut event_pump = sdl_context.event_pump()?;
//   let mut reader_id = Write::<GameEventsChannel>::fetch(&world).register_reader();
//   let mut slowdown_timer: Option<Duration> = None;
//   let frame_dt = Duration::new(0, 1_000_000_000u32 / 60);
//   let mut last_time = Instant::now();
//
//   'running: loop {
//     let current_time = Instant::now();
//     let mut frame_time = current_time - last_time;
//     last_time = current_time;
//
//     while frame_time.as_secs_f32() > 0.0 {
//       let dt = std::cmp::min(frame_time, frame_dt);
//
//       for event in world.read_resource::<GameEventsChannel>().read(&mut reader_id) {
//         if let GameEvents::PlayerDeath(_) = event {
//           slowdown_timer = Some(Duration::from_secs_f32(0.0));
//         }
//       }
//
//       if let Some(mut timer) = slowdown_timer.take() {
//         timer += dt;
//         if timer.as_secs_f32() <= SLOW_DOWN_DURATION_ON_DEATH {
//           let easing = ease_in_out_cubic(timer.as_secs_f32() / SLOW_DOWN_DURATION_ON_DEATH);
//           let slow_amount = (1.0 - easing) * 0.15 + easing * 1.0;
//           *world.write_resource() = Duration::from_secs_f32(dt.as_secs_f32() * slow_amount);
//           slowdown_timer.replace(timer);
//         }
//       } else {
//         *world.write_resource() = dt;
//       }
//
//       for event in event_pump.poll_iter() {
//         match event {
//           Event::Quit { .. }
//           | Event::KeyDown {
//             keycode: Some(Keycode::Escape),
//             ..
//           } => break 'running,
//           _ => {}
//         }
//       }
//
//       let keycodes = event_pump
//         .keyboard_state()
//         .pressed_scancodes()
//         .filter_map(Keycode::from_scancode)
//         .collect::<HashSet<Keycode>>();
//       *world.write_resource() = keycodes;
//
//       dispatcher.dispatch(&world);
//       world.maintain();
//
//       frame_time -= dt;
//     }
//
//     render::render(
//       &mut canvas,
//       Color::from(RGB_COLOR_BACKGROUND),
//       &textures,
//       world.system_data(),
//     )?;
//   }
//
//   drop(reader_id);
//
//   Ok(())
// }

mod color;
mod components;
mod easings;
mod environment;
mod events;
mod render;
mod resources;
mod systems;

use crate::{
  environment::{RGB_CLEAR_COLOR, SCREEN_RENDER_HEIGHT, SCREEN_RENDER_WIDTH},
  events::GameEvents,
  render::{calculate_size_for_quads, Gl},
  resources::{Camera, Circle, CircleGeometry, ProjectileSpawnConfig, Quad, Shake},
  systems::*,
};
use bevy_ecs::{event::Events, prelude::*, system::SystemState, world::World};
use lyon::tessellation::{FillTessellator, StrokeTessellator};
use render::{calculate_size_for_circles, create_draw_buffer};
use sdl2::{
  event::{Event, WindowEvent},
  keyboard::Keycode,
  video::GLProfile,
};
use std::{
  collections::HashSet,
  time::{Duration, Instant},
};
use systems::shooting_system;

fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let sdl_video = sdl_context.video()?;
  let gl_attr = sdl_video.gl_attr();
  gl_attr.set_context_profile(GLProfile::Core);
  gl_attr.set_context_version(3, 3);
  let sdl_window = sdl_video
    .window("bytepath", SCREEN_RENDER_WIDTH, SCREEN_RENDER_HEIGHT)
    .opengl()
    .resizable()
    .position_centered()
    .build()
    .map_err(|e| e.to_string())?;
  let _ctx = sdl_window.gl_create_context()?;
  let gl = Gl::load_with(|name| sdl_video.gl_get_proc_address(name) as *const _);
  debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
  debug_assert_eq!(gl_attr.context_version(), (3, 3));
  let mut opengl_ctx = render::init(&gl)?;

  let mut world = World::default();
  world.insert_resource(ProjectileSpawnConfig::default());
  world.insert_resource(HashSet::<Keycode>::default());
  world.insert_resource(Camera::default());
  world.insert_resource(Shake::default());
  world.insert_resource(Duration::default());
  world.insert_resource(Events::<GameEvents>::default());
  world.insert_resource(StrokeTessellator::new());
  world.insert_resource(FillTessellator::new());
  world.insert_resource(create_draw_buffer::<Circle>(
    &gl,
    &opengl_ctx,
    calculate_size_for_circles,
  ));
  world.insert_resource(create_draw_buffer::<Quad>(&gl, &opengl_ctx, calculate_size_for_quads));

  let mut render_state = SystemState::<render::RenderSystemState>::new(&mut world);

  let mut startup_schedule = Schedule::default();
  startup_schedule.add_stage(
    "startup",
    SystemStage::single_threaded().with_system(player_spawn_system),
  );

  let mut game_schedule = Schedule::default();
  game_schedule.add_stage("events", {
    let mut stage = SystemStage::parallel();
    stage.add_system(Events::<GameEvents>::update_system);
    stage
  });
  game_schedule.add_stage_after("events", "game", {
    let mut stage = SystemStage::parallel();
    stage.add_system(player_system);
    stage.add_system(camera_shake_system.after(player_system));
    stage.add_system(shooting_system.after(player_system));
    stage.add_system(projectile_spawn_system.after(player_system));
    stage.add_system(projectile_system.after(player_system));
    stage.add_system(projectile_death_system.after(projectile_system));

    stage
  });

  let mut event_pump = sdl_context.event_pump()?;
  let frame_dt = Duration::new(0, 1_000_000_000u32 / 60);
  let mut last_time = Instant::now();

  startup_schedule.run(&mut world);

  'running: loop {
    let current_time = Instant::now();
    let mut frame_time = current_time - last_time;
    last_time = current_time;

    while frame_time.as_secs_f32() > 0.0 {
      let dt = std::cmp::min(frame_time, frame_dt);

      *world.resource_mut() = dt;

      for event in event_pump.poll_iter() {
        match event {
          Event::Quit { .. }
          | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
          } => break 'running,
          Event::Window {
            win_event: WindowEvent::Resized(w, h),
            ..
          } => opengl_ctx.viewport = (w, h),
          _ => {}
        }
      }

      let keycodes = event_pump
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect::<HashSet<Keycode>>();
      *world.resource_mut() = keycodes;

      game_schedule.run(&mut world);

      frame_time -= dt;
    }

    render::render_gl(&gl, &opengl_ctx, render_state.get_mut(&mut world))?;

    sdl_window.gl_swap_window();
  }

  render::delete(&gl, &opengl_ctx, render_state.get_mut(&mut world));

  Ok(())
}
