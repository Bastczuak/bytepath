use ggez::*;

pub type RenderSystemData<'a> = ();

pub fn render(ctx: &mut Context, background: graphics::Color, data: RenderSystemData) -> GameResult {
  graphics::clear(ctx, background);
  let circle = ggez::graphics::Mesh::new_circle(
    ctx,
    graphics::DrawMode::stroke(1.0),
    glam::Vec2::new(0.0, 0.0),
    100.0,
    1.0,
    graphics::Color::WHITE,
  )?;
  graphics::draw(ctx, &circle, (glam::Vec2::new(380.0, 380.0),))?;
  graphics::present(ctx)?;
  Ok(())
}
