mod graphics_window;
mod rng_buffer;

use std::ops::Add;
use std::time::{Duration, Instant};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::WindowId;
use crate::graphics_window::{Color, GraphicsBuffer, GraphicsWindow, WindowConfig};
use crate::rng_buffer::RngBuffer;

fn main() {
    let config = WindowConfig {
        title: String::from("Test"),
        resizable: false,
        width: 800,
        height: 600,
    };
    let (mut graphics_window, event_loop) = GraphicsWindow::build(config);

    let mut rng_buffer = RngBuffer::new(100_000);
    rng_buffer.init(());

    event_loop.run(move |event, _, control_flow| {

        let next_tick = Instant::now().add(Duration::from_millis(1000/60));
        *control_flow = ControlFlow::WaitUntil(next_tick);

        match handle_event(&event) {
            EventResponse::Exit => { *control_flow = ControlFlow::Exit }
            EventResponse::RedrawRequested(_) => graphics_window.redraw(),
            EventResponse::Tick => {
                draw(&mut graphics_window.get_graphics(), &mut rng_buffer);
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

fn draw(graphics: &mut GraphicsBuffer, rng_buffer: &mut RngBuffer) {

    graphics.clear(Color::BLACK);


    // TODO ...


    for x in 0..graphics.get_width() {
        for y in 0..graphics.get_height() {
            let r = rng_buffer.next() as u8;
            let g = rng_buffer.next() as u8;
            let b = rng_buffer.next() as u8;
            graphics.draw_pixel(x, y, Color::from_rgb(r, g, b));
        }
    }

    /*
    for _i in 0..100000 {
        let x = rand::thread_rng().gen_range(0..buffer.width);
        let y = rand::thread_rng().gen_range(0..buffer.height);
        let r = rand::thread_rng().gen_range(0..255);
        let g = rand::thread_rng().gen_range(0..255);
        let b = rand::thread_rng().gen_range(0..255);
        buffer.draw_pixel(x, y, Color::from_rgb(r, g, b));
    }
    */
}
