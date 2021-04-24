mod components;
mod render;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;
use specs::prelude::*;
use crate::components::Position;

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

  let mut canvas = sdl_window.into_canvas().build().map_err(|e| e.to_string())?;
  canvas
    .set_logical_size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .map_err(|e| e.to_string())?;
  canvas.set_draw_color(Color::BLACK);
  canvas.clear();
  canvas.present();

  let mut dispatcher = DispatcherBuilder::new().build();
  let mut world = World::new();
  dispatcher.setup(&mut world);
  render::RenderSystemData::setup(&mut world);
  world
    .create_entity()
    .with(Position{ x: (SCREEN_WIDTH / 2) as i16, y: (SCREEN_HEIGHT / 2) as i16 })
    .build();

  let mut event_pump = sdl_context.event_pump()?;

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        _ => {}
      }
    }

    dispatcher.dispatch(&mut world);
    world.maintain();
    render::render(&mut canvas, Color::BLACK, world.system_data())?;

    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }

  Ok(())
}
