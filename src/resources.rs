#[derive(Default)]
pub struct DeltaTick(pub u32);

impl DeltaTick {
  pub fn in_seconds(&self) -> f32 {
    self.0 as f32 / 1000.0
  }
}

#[derive(Default)]
pub struct Shake {
  pub is_shaking: bool,
  pub x: i32,
  pub y: i32,
}
