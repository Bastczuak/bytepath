mod environment;
mod render;
mod systems;

use crate::{
  environment::{RGB_COLOR_BACKGROUND, SCREEN_HEIGHT, SCREEN_WIDTH},
  systems::HelloWorldSystem,
};
use ggez::*;
use specs::prelude::*;

fn main() -> GameResult {
  let cb = ContextBuilder::new("bytepath", "bastczuak")
    .window_mode(conf::WindowMode {
      width: SCREEN_WIDTH,
      height: SCREEN_HEIGHT,
      fullscreen_type: conf::FullscreenType::Desktop,
      ..Default::default()
    })
    .window_setup(conf::WindowSetup {
      title: String::from("Bytepath"),
      ..Default::default()
    });
  let (mut ctx, event_loop) = cb.build()?;

  graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);
  let window_color_format = graphics::get_window_color_format(&ctx);
  let canvas = graphics::Canvas::new(
    &mut ctx,
    SCREEN_WIDTH as u16,
    SCREEN_HEIGHT as u16,
    conf::NumSamples::One,
    window_color_format,
  )?;

  let mut dispatcher = DispatcherBuilder::new().with(HelloWorldSystem, "hello", &[]).build();
  let mut world = World::new();
  dispatcher.setup(&mut world);
  render::RenderSystemData::setup(&mut world);

  event_loop.run(move |mut event, _window_target, control_flow| {
    if !ctx.continuing {
      *control_flow = winit::event_loop::ControlFlow::Exit;
      return;
    }

    *control_flow = winit::event_loop::ControlFlow::Poll;
    event::process_event(&mut ctx, &mut event);

    match event {
      event::winit_event::Event::WindowEvent { event, .. } => match event {
        event::winit_event::WindowEvent::CloseRequested => event::quit(&mut ctx),
        event::winit_event::WindowEvent::KeyboardInput {
          input:
            event::winit_event::KeyboardInput {
              virtual_keycode: Some(keycode),
              ..
            },
          ..
        } => {
          if let event::KeyCode::Escape = keycode {
            *control_flow = winit::event_loop::ControlFlow::Exit
          }
        }
        _ => {}
      },
      event::winit_event::Event::MainEventsCleared => {
        ctx.timer_context.tick();
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(&mut ctx, DESIRED_FPS) {
          let keycodes = input::keyboard::pressed_keys(&mut ctx).clone();
          *world.write_resource() = keycodes;
          dispatcher.dispatch(&world);
          world.maintain();
        }
        timer::yield_now();

        render::render(
          &mut ctx,
          &canvas,
          graphics::Color::from(RGB_COLOR_BACKGROUND),
          world.system_data(),
        )
        .expect("Error while rendering!");
      }
      _ => {}
    }
  })
}
