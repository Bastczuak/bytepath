pub struct ColorGl {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl ColorGl {
  pub fn to_array(&self) -> [f32; 4] {
    [self.r, self.g, self.g, self.a]
  }
}

impl From<(u8, u8, u8)> for ColorGl {
  fn from((r, g, b): (u8, u8, u8)) -> ColorGl {
    ColorGl {
      r: r as f32 / 255.0,
      g: g as f32 / 255.0,
      b: b as f32 / 255.0,
      a: 1.0,
    }
  }
}
