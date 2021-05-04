mod components;
mod render;
mod resources;
mod systems;

use crate::components::{Angle, Player, Transform, Velocity};
use crate::resources::{DeltaTick, Direction, MovementCommand, Shake};
use crate::systems::{PlayerSystem, ShakeSystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use specs::prelude::*;
use std::time::Duration;

const SCREEN_WIDTH: u32 = 480;
const SCREEN_HEIGHT: u32 = 280;

fn main() -> Result<(), String> {
  let sdl_context = sdl2::init()?;
  let sdl_video = sdl_context.video()?;
  let display_mode = sdl_video.desktop_display_mode(0)?;
  let factor = std::cmp::min(
    display_mode.w / SCREEN_WIDTH as i32,
    display_mode.h / SCREEN_HEIGHT as i32,
  ) as u32;
  let sdl_window = sdl_video
    .window("bytepath", SCREEN_WIDTH * factor, SCREEN_HEIGHT * factor)
    .position_centered()
    .build()
    .map_err(|e| e.to_string())?;

  let mut canvas = sdl_window
    .into_canvas()
    .accelerated()
    .present_vsync()
    .build()
    .map_err(|e| e.to_string())?;
  canvas
    .set_logical_size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .map_err(|e| e.to_string())?;
  canvas.set_draw_color(Color::BLACK);
  canvas.clear();
  canvas.present();

  let texture_creator = canvas.texture_creator();
  let mut texture = texture_creator
    .create_texture_target(PixelFormatEnum::RGBA8888, SCREEN_WIDTH, SCREEN_HEIGHT)
    .map_err(|e| e.to_string())?;

  let mut dispatcher = DispatcherBuilder::new()
    .with(ShakeSystem::new(), "shake_system", &[])
    .with(PlayerSystem, "player_system", &[])
    .build();
  let mut world = World::new();
  world.register::<Player>();
  world.register::<Transform>();
  let movement_command: Option<MovementCommand> = None;
  world.insert(movement_command);
  dispatcher.setup(&mut world);
  render::RenderSystemData::setup(&mut world);

  world
    .create_entity()
    .with(Player)
    .with(Transform::new((SCREEN_WIDTH / 2) as i16, (SCREEN_HEIGHT / 2) as i16))
    .with(Angle::default())
    .with(Velocity::default())
    .build();

  let sdl_timer = sdl_context.timer()?;
  let mut last_tick = 0;

  let mut event_pump = sdl_context.event_pump()?;

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        Event::KeyDown {
          keycode: Some(Keycode::Space),
          ..
        } => world.write_resource::<Shake>().is_shaking = true,
        Event::KeyDown {
          keycode: Some(Keycode::A),
          ..
        }
        | Event::KeyDown {
          keycode: Some(Keycode::Left),
          ..
        } => *world.write_resource() = Some(MovementCommand::Move(Direction::Left)),
        Event::KeyDown {
          keycode: Some(Keycode::D),
          ..
        }
        | Event::KeyDown {
          keycode: Some(Keycode::Right),
          ..
        } => *world.write_resource() = Some(MovementCommand::Move(Direction::Right)),
        Event::KeyUp {
          keycode: Some(Keycode::A),
          repeat: false,
          ..
        }
        | Event::KeyUp {
          keycode: Some(Keycode::D),
          repeat: false,
          ..
        }
        | Event::KeyUp {
          keycode: Some(Keycode::Left),
          ..
        }
        | Event::KeyUp {
          keycode: Some(Keycode::Right),
          ..
        } => *world.write_resource() = Some(MovementCommand::Stop),
        _ => {}
      }
    }

    {
      let current_tick = sdl_timer.ticks();
      *world.write_resource() = DeltaTick(current_tick - last_tick);
      last_tick = current_tick;
    }

    dispatcher.dispatch(&world);
    world.maintain();

    render::render(&mut canvas, Color::BLACK, &mut texture, world.system_data())?;

    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }

  Ok(())
}
