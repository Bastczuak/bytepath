pub type EasingFunction = fn(f32) -> f32;

pub fn ease_in_out_cubic(x: f32) -> f32 {
  if x < 0.5 {
    4.0 * x * x * x
  } else {
    1.0 - f32::powf(-2.0 * x + 2.0, 3.0) / 2.0
  }
}
