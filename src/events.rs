use bevy_ecs::event::Event;

#[derive(Event)]
pub enum GameEvents {
  PlayerDeath,
}
