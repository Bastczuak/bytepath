use specs::{prelude::*, Component};

#[derive(Component, Default)]
#[storage(DenseVecStorage)]
pub struct Position {
  pub x: i16,
  pub y: i16,
}
