#[derive(Default)]
pub struct DeltaTick(pub u32);

#[derive(Default)]
pub struct Shake {
  pub is_shaking: bool,
  pub x: i32,
  pub y: i32,
}
