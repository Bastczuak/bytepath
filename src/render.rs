use crate::components::Position;
use crate::resources::Shake;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

pub type RenderSystemData<'a> = (ReadStorage<'a, Position>, Read<'a, Shake>);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  texture: &mut Texture,
  data: RenderSystemData,
) -> Result<(), String> {
  let (pos, shake) = data;
  canvas
    .with_texture_canvas(texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::BLACK);
      texture_canvas.clear();
      for pos in (&pos).join() {
        texture_canvas.circle(pos.x, pos.y, 50, Color::WHITE).unwrap();
      }
    })
    .map_err(|e| e.to_string())?;

  canvas.set_draw_color(background);
  canvas.clear();
  if shake.is_shaking {
    canvas.copy(&texture, None, Rect::new(shake.x, shake.y, 480, 280))?;
  } else {
    canvas.copy(&texture, None, None)?;
  }
  canvas.present();

  Ok(())
}
