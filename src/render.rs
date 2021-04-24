use specs::prelude::*;
use crate::components::Position;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::gfx::primitives::DrawRenderer;

pub type RenderSystemData<'a> = (
  ReadStorage<'a, Position>,
);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  data: RenderSystemData,
) -> Result<(), String> {
  canvas.set_draw_color(background);
  canvas.clear();

  for pos in (&data.0).join() {
    canvas.circle(pos.x, pos.y, 50, Color::WHITE)?;
  }

  canvas.present();

  Ok(())
}
