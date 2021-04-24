use crate::components::Position;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

pub type RenderSystemData<'a> = (ReadStorage<'a, Position>,);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  texture: &mut Texture,
  data: RenderSystemData,
) -> Result<(), String> {
  canvas
    .with_texture_canvas(texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::BLACK);
      texture_canvas.clear();
      for pos in (&data.0).join() {
        texture_canvas.circle(pos.x, pos.y, 50, Color::WHITE).unwrap();
      }
    })
    .map_err(|e| e.to_string())?;

  canvas.set_draw_color(background);
  canvas.clear();
  canvas.copy(&texture, None, None)?;

  canvas.present();

  Ok(())
}
