use specs::WorldExt;

struct HelloWorldSystem;

impl <'a>specs::System<'a> for HelloWorldSystem {
  type SystemData = ();

  fn run(&mut self, _data: Self::SystemData) {
    println!("Hello World");
  }
}

fn main() -> ggez::GameResult {
  let cb = ggez::ContextBuilder::new("bytepath", "bastczuak");
  let (mut ctx, event_loop) = cb.build()?;

  let mut dispatcher = specs::DispatcherBuilder::new()
    .with(HelloWorldSystem, "hello", &[])
    .build();
  let mut world = specs::World::new();
  dispatcher.setup(&mut world);
  // render::RenderSystemData::setup(&mut world);


  event_loop.run(move |mut event, _window_target, control_flow| {
    if !ctx.continuing {
      *control_flow = ggez::winit::event_loop::ControlFlow::Exit;
      return;
    }

    *control_flow = ggez::winit::event_loop::ControlFlow::Poll;
    ggez::event::process_event(&mut ctx, &mut event);

    match event {
      ggez::event::winit_event::Event::WindowEvent { event, .. } => match event {
        ggez::event::winit_event::WindowEvent::CloseRequested => ggez::event::quit(&mut ctx),
        _ => {}
      },
      ggez::event::winit_event::Event::MainEventsCleared => {
        ctx.timer_context.tick();
        println!("{:?}", ggez::timer::fps(&ctx));
        dispatcher.dispatch(&world);
        world.maintain();
        ggez::timer::yield_now();
      }
      _ => {}
    }
  })
}
