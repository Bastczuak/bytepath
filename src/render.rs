use crate::components::{Animation, Position, Sprite};
use crate::resources::Shake;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

pub type RenderSystemData<'a> = (
  Read<'a, Shake>,
  ReadStorage<'a, Position>,
  ReadStorage<'a, Sprite>,
  ReadStorage<'a, Animation>,
);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  textures: &[Texture],
  data: RenderSystemData,
) -> Result<(), String> {
  let (shake, positions, sprites, animations) = data;
  canvas.set_draw_color(background);
  canvas.clear();

  for (position, sprite) in (&positions, &sprites).join() {
    let screen_position = Point::new(position.x as i32 + shake.x, position.y as i32 + shake.y);
    let screen_rect = Rect::from_center(screen_position, sprite.region.width(), sprite.region.height());
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
      let screen_rect = Rect::from_center(screen_position, sprite.region.width(), sprite.region.height());
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

  canvas.present();

  Ok(())
}
