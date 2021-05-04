use crate::components::{Angle, Transform};
use crate::resources::Shake;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use specs::prelude::*;

pub type RenderSystemData<'a> = (Read<'a, Shake>, ReadStorage<'a, Transform>, ReadStorage<'a, Angle>);

pub fn render(
  canvas: &mut WindowCanvas,
  background: Color,
  texture: &mut Texture,
  data: RenderSystemData,
) -> Result<(), String> {
  let (shake, transforms, angles) = data;
  canvas
    .with_texture_canvas(texture, |texture_canvas| {
      texture_canvas.set_draw_color(Color::BLACK);
      texture_canvas.clear();
      for (transform, angle) in (&transforms, &angles).join() {
        texture_canvas
          .circle(
            transform.translation.x as i16,
            transform.translation.y as i16,
            15,
            Color::WHITE,
          )
          .unwrap();
        // todo: just debug
        texture_canvas
          .line(
            transform.translation.x as i16,
            transform.translation.y as i16,
            (transform.translation.x + 2.0 * 15.0 * f32::cos(angle.radians)) as i16,
            (transform.translation.y + 2.0 * 15.0 * f32::sin(angle.radians)) as i16,
            Color::WHITE,
          )
          .unwrap();
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
