mod graphics_window;
mod rng_buffer;
mod matrix_test;
mod world;
mod fps_counter;
mod grid;

use std::ops::Add;
use std::time::{Duration, Instant};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::WindowId;
use crate::fps_counter::FpsCounter;
use crate::graphics_window::{GraphicsWindow, WindowConfig};
use crate::rng_buffer::RngBuffer;
use crate::world::World;

fn main() {
    let target_fps = 1000;
    let target_frame_time: Duration = Duration::from_millis(1000 / target_fps);
    let mut next_tick = Instant::now().add(target_frame_time);

    let mut fps_counter = FpsCounter::every_32_frames();

    let mut rng = RngBuffer::with_capacity(1_000);

    let mut world = World::new(800, 600);
    world.load(&mut rng);

    /*
    match matrix_test::run() {
        None => exit(0),
        Some(_) => {},
    }
    */



    let config = WindowConfig {
        title: String::from("Test"),
        resizable: false,
        width: 800,
        height: 600,
    };
    let (mut graphics_window, event_loop) = GraphicsWindow::build(config);

    event_loop.run(move |event, _, control_flow| {

        if Instant::now() > next_tick {
            next_tick = next_tick.add(target_frame_time);
            *control_flow = ControlFlow::WaitUntil(next_tick);
        }

        match handle_event(&event) {
            EventResponse::Exit => { *control_flow = ControlFlow::Exit }
            EventResponse::RedrawRequested(_) => graphics_window.redraw(),
            EventResponse::Tick => {
                fps_counter.tick();
                world::step(&mut world, &mut rng);
                world.draw(&mut graphics_window.get_graphics());
                graphics_window.get_window().request_redraw();
            }
            EventResponse::None => {}
        }
    });
}

fn handle_event(event: &Event<()>) -> EventResponse {
    match event {
        Event::RedrawRequested(window_id) => EventResponse::RedrawRequested(*window_id),
        Event::WindowEvent { event: window_event, .. } => handle_window_event(window_event),
        Event::NewEvents(StartCause::ResumeTimeReached { .. }) => EventResponse::Tick,
        _ => EventResponse::None,
    }
}

fn handle_window_event(event: &WindowEvent) -> EventResponse {
    match event {
        WindowEvent::CloseRequested => { return EventResponse::Exit }
        WindowEvent::KeyboardInput { .. } => EventResponse::None,
        _ => EventResponse::None,
    }
}

enum EventResponse {
    None, Exit, RedrawRequested(WindowId), Tick
}
