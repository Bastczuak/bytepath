use crate::{
  components::{Animation, LineParticle, Position, Sprite},
  resources::{Flash, Shake},
};
use sdl2::{
  gfx::primitives::DrawRenderer,
  pixels::Color,
  rect::{Point, Rect},
  render::{Texture, WindowCanvas},
};
use specs::prelude::*;

pub type RenderSystemData<'a> = (
  Read<'a, Shake>,
  Read<'a, Flash>,
  ReadStorage<'a, Position>,
  ReadStorage<'a, Sprite>,
  ReadStorage<'a, Animation>,
  ReadStorage<'a, LineParticle>,
);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  textures: &[Texture],
  data: RenderSystemData,
) -> Result<(), String> {
  let (shake, flash, positions, sprites, animations, line_particles) = data;

  if flash.0 > 0 {
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();
    return Ok(());
  }

  canvas.set_draw_color(background);
  canvas.clear();

  for (position, sprite) in (&positions, &sprites).join() {
    let screen_position = Point::new(position.x as i32 + shake.x, position.y as i32 + shake.y);
    let screen_rect = Rect::from_center(
      screen_position,
      sprite.scaled_region_width(),
      sprite.scaled_region_height(),
    );
    canvas.copy_ex(
      &textures[sprite.texture_idx],
      sprite.region,
      screen_rect,
      sprite.rotation,
      None,
      false,
      false,
    )?;
  }

  for (position, animation) in (&positions, &animations).join() {
    if let Some(sprite) = animation.current_frame() {
      let screen_position = Point::new(position.x as i32, position.y as i32);
      let screen_rect = Rect::from_center(
        screen_position,
        sprite.scaled_region_width(),
        sprite.scaled_region_height(),
      );
      canvas.copy_ex(
        &textures[sprite.texture_idx],
        sprite.region,
        screen_rect,
        sprite.rotation,
        None,
        false,
        false,
      )?;
    }
  }

  for particle in (&line_particles).join() {
    canvas.thick_line(
      particle.x1 as i16 + shake.x as i16,
      particle.y1 as i16 + shake.y as i16,
      particle.x2 as i16 + shake.x as i16,
      particle.y2 as i16 + shake.y as i16,
      particle.width as u8,
      particle.color,
    )?;
  }

  canvas.present();

  Ok(())
}
