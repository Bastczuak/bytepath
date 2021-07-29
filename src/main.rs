mod components;
mod easings;
mod render;
mod resources;
mod systems;

use crate::{
  components::{Angle, Interpolation, Player, Position, ShootingEffect, Sprite, Velocity},
  resources::DeltaTick,
  systems::{PlayerDeathSystem, PlayerSystem, ProjectileDeathSystem, ProjectileSystem, ShakeSystem, ShootingSystem},
};
use sdl2::{
  event::Event,
  gfx::primitives::DrawRenderer,
  keyboard::Keycode,
  pixels::Color,
  rect::Rect,
  render::{BlendMode, Texture, TextureCreator, WindowCanvas},
  video::WindowContext,
};
use specs::prelude::*;
use std::{collections::HashSet, time::Duration};

pub const SCREEN_WIDTH: u32 = 480;
pub const SCREEN_HEIGHT: u32 = 280;

fn create_ship_texture<'a, 'b>(
  texture_creator: &'a TextureCreator<WindowContext>,
  canvas: &'b mut WindowCanvas,
) -> Result<Texture<'a>, String> {
  let mut texture = texture_creator
    .create_texture_target(texture_creator.default_pixel_format(), 32, 32)
    .map_err(|e| e.to_string())?;
  canvas
    .with_texture_canvas(&mut texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
      texture_canvas.clear();
      texture_canvas.circle(16, 16, 15, Color::WHITE).unwrap();
    })
    .map_err(|e| e.to_string())?;
  texture.set_blend_mode(BlendMode::Blend);

  Ok(texture)
}

fn create_shooting_effect_texture<'a, 'b>(
  texture_creator: &'a TextureCreator<WindowContext>,
  canvas: &'b mut WindowCanvas,
) -> Result<Texture<'a>, String> {
  let mut texture = texture_creator
    .create_texture_target(texture_creator.default_pixel_format(), 8, 8)
    .map_err(|e| e.to_string())?;
  canvas
    .with_texture_canvas(&mut texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
      texture_canvas.clear();
      texture_canvas.set_draw_color(Color::WHITE);
      texture_canvas.fill_rect(Rect::new(0, 0, 8, 8)).unwrap();
    })
    .map_err(|e| e.to_string())?;
  texture.set_blend_mode(BlendMode::Blend);

  Ok(texture)
}

fn create_projectile_texture<'a, 'b>(
  texture_creator: &'a TextureCreator<WindowContext>,
  canvas: &'b mut WindowCanvas,
) -> Result<Texture<'a>, String> {
  let mut texture = texture_creator
    .create_texture_target(texture_creator.default_pixel_format(), 8, 8)
    .map_err(|e| e.to_string())?;
  canvas
    .with_texture_canvas(&mut texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
      texture_canvas.clear();
      texture_canvas.circle(4, 4, 3, Color::WHITE).unwrap();
    })
    .map_err(|e| e.to_string())?;
  texture.set_blend_mode(BlendMode::Blend);

  Ok(texture)
}

fn create_projectile_death_texture<'a, 'b>(
  texture_creator: &'a TextureCreator<WindowContext>,
  canvas: &'b mut WindowCanvas,
) -> Result<Texture<'a>, String> {
  let mut texture = texture_creator
    .create_texture_target(texture_creator.default_pixel_format(), 6, 6)
    .map_err(|e| e.to_string())?;
  canvas
    .with_texture_canvas(&mut texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
      texture_canvas.clear();
      texture_canvas.set_draw_color(Color::WHITE);
      texture_canvas.fill_rect(Rect::new(0, 0, 6, 3)).unwrap();
      texture_canvas.set_draw_color(Color::RGB(241, 103, 69));
      texture_canvas.fill_rect(Rect::new(0, 3, 6, 6)).unwrap();
    })
    .map_err(|e| e.to_string())?;
  texture.set_blend_mode(BlendMode::Blend);

  Ok(texture)
}

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
  let textures = [
    create_ship_texture(&texture_creator, &mut canvas)?,
    create_shooting_effect_texture(&texture_creator, &mut canvas)?,
    create_projectile_texture(&texture_creator, &mut canvas)?,
    create_projectile_death_texture(&texture_creator, &mut canvas)?,
  ];

  let mut dispatcher = DispatcherBuilder::new()
    .with(ShakeSystem::default(), "shake_system", &[])
    .with(PlayerSystem, "player_system", &[])
    .with(ShootingSystem, "shooting_system", &["player_system"])
    .with(ProjectileSystem::default(), "projectile_system", &["player_system"])
    .with(ProjectileDeathSystem, "projectile_death_system", &["projectile_system"])
    .with(PlayerDeathSystem::default(), "player_death_system", &["player_system"])
    .build();
  let mut world = World::new();
  dispatcher.setup(&mut world);
  render::RenderSystemData::setup(&mut world);

  world
    .create_entity()
    .with(Player)
    .with(Position {
      x: SCREEN_WIDTH as f32 / 2.0,
      y: SCREEN_HEIGHT as f32 / 2.0,
    })
    .with(Angle::default())
    .with(Velocity::default())
    .with(Sprite {
      texture_idx: 0,
      region: Rect::new(0, 0, 32, 32),
      rotation: 0.0,
    })
    .build();
  world
    .create_entity()
    .with(ShootingEffect)
    .with(Position {
      x: SCREEN_WIDTH as f32 / 2.0,
      y: SCREEN_HEIGHT as f32 / 2.0,
    })
    .with(Sprite {
      texture_idx: 1,
      region: Rect::new(0, 0, 0, 0),
      rotation: 45.0,
    })
    .with(Interpolation::new(vec![(8.0, 0.0)], 0.2))
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
        _ => {}
      }
    }

    {
      let keycodes = event_pump
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect::<HashSet<Keycode>>();
      *world.write_resource() = keycodes;
    }

    {
      let current_tick = sdl_timer.ticks();
      *world.write_resource() = DeltaTick(current_tick - last_tick);
      last_tick = current_tick;
    }

    dispatcher.dispatch(&world);
    world.maintain();
    render::render(&mut canvas, Color::BLACK, &textures, world.system_data())?;

    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }

  Ok(())
}
