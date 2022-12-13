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
  render::{calculate_size_for_lines, calculate_size_for_quads, Gl},
  resources::*,
  systems::*,
};
use bevy_ecs::{event::Events, prelude::*, system::SystemState, world::World};
use lyon::tessellation::{FillTessellator, StrokeTessellator};
use rand::SeedableRng;
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
  world.insert_resource(Time::default());
  world.insert_resource(rand::rngs::SmallRng::from_entropy());
  world.insert_resource(EntitySpawnTimer::default());
  world.insert_resource(HashSet::<Keycode>::default());
  world.insert_resource(Camera::default());
  world.insert_resource(Shake::default());
  world.insert_resource(Flash::default());
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
  world.insert_resource(create_draw_buffer::<Line>(&gl, &opengl_ctx, calculate_size_for_lines));

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
    stage.add_system(timing_system.after(Events::<GameEvents>::update_system));

    stage
  });
  game_schedule.add_stage_after("events", "game", {
    let mut stage = SystemStage::parallel();
    stage.add_system(player_system);
    stage.add_system(shooting_system.after(player_system));
    stage.add_system(tick_effect_spawn_system.after(player_system));
    stage.add_system(tick_effect_system.after(player_system));
    stage.add_system(projectile_spawn_system.after(player_system));
    stage.add_system(projectile_system.after(player_system));
    stage.add_system(projectile_death_system.after(projectile_system));
    stage.add_system(player_explosion_spawn_system.after(player_system));
    stage.add_system(trail_effect_spawn_system.after(player_system));
    stage.add_system(ammo_pickup_system.after(player_system));
    stage.add_system(boost_pickup_system.after(player_system));
    stage.add_system(trail_effect_system.after(trail_effect_spawn_system));
    stage.add_system(camera_shake_system);
    stage.add_system(screen_flash_system);
    stage.add_system(ammo_pickup_spawn_system);
    stage.add_system(explosion_system);
    stage.add_system(boost_pickup_spawn_system);

    stage
  });

  startup_schedule.run(&mut world);

  let frame_dt = Duration::new(0, 1_000_000_000u32 / 60);
  let mut last_time = Instant::now();
  let mut event_pump = sdl_context.event_pump()?;

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
