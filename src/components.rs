use specs::{prelude::*, Component};

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct Position {
  pub x: i16,
  pub y: i16,
}
