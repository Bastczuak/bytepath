use crate::components::Position;
use specs::shrev::EventChannel;

pub enum GameEvents {
  PlayerDeath(Position),
  PlayerSpawn,
  ProjectileDeath(Position),
}

pub type GameEventsChannel = EventChannel<GameEvents>;

#[derive(Default)]
pub struct Shake {
  pub x: i32,
  pub y: i32,
}

#[derive(Default)]
pub struct Flash(pub u8);
