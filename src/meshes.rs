use crate::environment::{RGB_COLOR_AMMUNITION, SCREEN_HEIGHT, SCREEN_WIDTH};
use ggez::*;

pub fn create_ammunition_mesh_batch(ctx: &mut Context) -> GameResult<graphics::MeshBatch> {
  let mesh = graphics::MeshBuilder::new()
    .rectangle(
      graphics::DrawMode::stroke(1.0),
      graphics::Rect::new(0.0, 0.0, 8.0, 8.0),
      graphics::Color::from(RGB_COLOR_AMMUNITION),
    )?
    .build(ctx)?;

  let mesh_batch = graphics::MeshBatch::new(mesh)?;

  Ok(mesh_batch)
}

pub fn create_player_mesh_batch(ctx: &mut Context) -> GameResult<graphics::MeshBatch> {
  let mesh = graphics::MeshBuilder::new()
    .circle(
      graphics::DrawMode::stroke(1.0),
      glam::Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
      8.0,
      1.0,
      graphics::Color::WHITE,
    )?
    .build(ctx)?;

  let mesh_batch = graphics::MeshBatch::new(mesh)?;

  Ok(mesh_batch)
}
