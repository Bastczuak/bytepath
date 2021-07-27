use crate::components::Position;
use specs::shrev::EventChannel;

#[derive(Default)]
pub struct DeltaTick(pub u32);

impl DeltaTick {
  pub fn in_seconds(&self) -> f32 {
    self.0 as f32 / 1000.0
  }
}

pub enum GameEvents {
  PlayerDeath(Position),
}

pub type GameEventsChannel = EventChannel<GameEvents>;

#[derive(Default)]
pub struct Shake {
  pub x: i32,
  pub y: i32,
}
