use ggez::*;
use specs::prelude::*;

pub struct HelloWorldSystem;

impl<'a> specs::System<'a> for HelloWorldSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, std::collections::HashSet<ggez::winit::event::VirtualKeyCode>>,
  );

  fn run(&mut self, data: Self::SystemData) {
    let (_, keycodes) = data;
    if keycodes.contains(&ggez::winit::event::VirtualKeyCode::Space) {
      println!("space");
    }
  }
}
