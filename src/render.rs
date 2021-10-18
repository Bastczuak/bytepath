use crate::{
  components::{Ammunition, Player, Position},
  meshes::{create_ammunition_mesh_batch, create_player_mesh_batch},
};
use ggez::*;
use ggez::graphics::draw;
use specs::{Join, ReadStorage};
use crate::components::Angle;

pub type RenderSystemData<'a> = (
  ReadStorage<'a, Ammunition>,
  ReadStorage<'a, Player>,
  ReadStorage<'a, Position>,
  ReadStorage<'a, Angle>,
);

pub fn render(
  ctx: &mut Context,
  canvas: &graphics::Canvas,
  background: graphics::Color,
  data: RenderSystemData,
) -> GameResult {
  let (ammunitions, players, positions, angles) = data;

  graphics::set_canvas(ctx, Some(canvas));
  graphics::clear(ctx, background);

  let mut mesh_batch = create_ammunition_mesh_batch(ctx)?;
  for (_, position) in (&ammunitions, &positions).join() {
    let p = graphics::DrawParam::new()
      .dest(glam::Vec2::new(position.x, position.y))
      .rotation(45.0);
    mesh_batch.add(p);
  }

  for (_, position, angle) in (&players, &positions, &angles).join() {
    let mesh = graphics::MeshBuilder::new()
      .circle(
        graphics::DrawMode::stroke(1.0),
        glam::Vec2::new(0.0, 0.0),
        16.0,
        1.0,
        graphics::Color::WHITE,
      )?
      .build(ctx)?;
    draw(ctx, &mesh, graphics::DrawParam::new()
      .dest(glam::Vec2::new(position.x, position.y))
      .rotation((angle.radians * 180.0 / std::f32::consts::PI)))?;
  }

  mesh_batch.draw(ctx, graphics::DrawParam::default())?;
  graphics::set_canvas(ctx, None);
  graphics::draw(ctx, canvas, graphics::DrawParam::default())?;
  graphics::present(ctx)?;
  Ok(())
}
