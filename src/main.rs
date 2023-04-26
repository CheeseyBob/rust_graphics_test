mod graphics_window;
mod rng_buffer;

use std::ops::{Add};
use std::time::{Duration, Instant};
use winit::dpi::{PhysicalSize};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;
use crate::graphics_window::{Color, GraphicsBuffer, GraphicsWindow};
use crate::rng_buffer::RngBuffer;

fn main() {
    let event_loop = EventLoop::new();

    let mut graphics_window = GraphicsWindow::build(&event_loop);
    graphics_window.window.set_title("Test");
    graphics_window.window.set_resizable(false);
    graphics_window.window.set_inner_size(PhysicalSize::new(800, 600));

    let mut rng_buffer = RngBuffer::new(100_000);
    rng_buffer.init(());


    event_loop.run(move |event, _, control_flow| {

        //*control_flow = ControlFlow::Wait;

        //dbg!(&event);

        let next_tick = Instant::now().add(Duration::from_millis(1000/60));
        *control_flow = ControlFlow::WaitUntil(next_tick);

        match handle_event(&event) {
            EventResponse::Exit => { *control_flow = ControlFlow::Exit }
            EventResponse::RedrawRequested(_) => graphics_window.redraw(),
            EventResponse::Tick => {
                draw(&mut graphics_window.graphics_buffer, &mut rng_buffer);
                graphics_window.window.request_redraw();
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

fn draw(buffer: &mut GraphicsBuffer, rng_buffer: &mut RngBuffer) {

    buffer.clear(Color::BLACK);


    // TODO ...


    for x in 0..buffer.get_width() {
        for y in 0..buffer.get_height() {
            let r = rng_buffer.next() as u8;
            let g = rng_buffer.next() as u8;
            let b = rng_buffer.next() as u8;
            buffer.draw_pixel(x, y, Color::from_rgb(r, g, b));
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
