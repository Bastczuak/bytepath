use crate::environment::{SCREEN_HEIGHT, SCREEN_WIDTH};
use ggez::*;

pub type RenderSystemData<'a> = ();

pub fn render(
  ctx: &mut Context,
  canvas: &graphics::Canvas,
  background: graphics::Color,
  data: RenderSystemData,
) -> GameResult {
  graphics::set_canvas(ctx, Some(canvas));
  graphics::clear(ctx, background);
  let rect = graphics::Mesh::new_rectangle(
    ctx,
    graphics::DrawMode::stroke(1.1),
    graphics::Rect::new(0.0, 0.0, 10.0, 10.0),
    graphics::Color::WHITE,
  )?;
  graphics::draw(
    ctx,
    &rect,
    (
      glam::Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
      45.0,
      graphics::Color::WHITE,
    ),
  )?;
  graphics::set_canvas(ctx, None);
  graphics::draw(
    ctx,
    canvas,
    graphics::DrawParam::new().color(graphics::Color::from((255, 255, 255))),
  )?;
  graphics::present(ctx)?;
  Ok(())
}
