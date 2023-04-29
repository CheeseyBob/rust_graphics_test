mod graphics_window;
mod rng_buffer;
mod matrix_test;

use std::ops::Add;
use std::process::exit;
use std::time::{Duration, Instant};
use rand::Rng;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::WindowId;
use crate::graphics_window::{Color, GraphicsBuffer, GraphicsWindow, WindowConfig};
use crate::rng_buffer::RngBuffer;

struct World {
    data: Vec<(usize, usize)>,
    width: usize,
    height: usize,
    rng_buffer: RngBuffer,
}

impl World {
    fn new(width: usize, height: usize) -> World {
        World {
            data: Vec::from([(99, 99), (99, 99), (99, 99)]),
            width,
            height,
            rng_buffer: RngBuffer::init_new(100_000, ())
        }
    }

    fn draw(&self, graphics: &mut GraphicsBuffer) {
        graphics.clear(Color::BLACK);

        for object in &self.data {
            let (x, y) = object;
            graphics.draw_pixel(*x, *y, Color::WHITE);
        }
    }

    fn step(&mut self) {
        for i in 0..self.data.len() {
            let (mut x, mut y) = self.data[i];

            let dx: isize = rand::thread_rng().gen_range(0..3) - 1;
            let dy: isize = rand::thread_rng().gen_range(0..3) - 1;

            match x.checked_add_signed(dx) {
                Some(new_x) if new_x < self.width => { x = new_x }
                _ => {}
            }
            match y.checked_add_signed(dy) {
                Some(new_y) if new_y < self.height => { y = new_y }
                _ => {}
            }

            self.data[i] = (x, y);
        }
    }
}

fn main() {

    let mut object_grid = World::new(256, 256);


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

    let mut rng_buffer = RngBuffer::new(100_000);
    rng_buffer.init(());

    event_loop.run(move |event, _, control_flow| {

        let next_tick = Instant::now().add(Duration::from_millis(1000/60));
        *control_flow = ControlFlow::WaitUntil(next_tick);

        match handle_event(&event) {
            EventResponse::Exit => { *control_flow = ControlFlow::Exit }
            EventResponse::RedrawRequested(_) => graphics_window.redraw(),
            EventResponse::Tick => {
                object_grid.step();
                object_grid.draw(&mut graphics_window.get_graphics());
                //draw_noise(&mut graphics_window.get_graphics(), &mut rng_buffer);
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

fn draw_noise(graphics: &mut GraphicsBuffer, rng_buffer: &mut RngBuffer) {
    graphics.clear(Color::BLACK);

    for x in 0..graphics.get_width() {
        for y in 0..graphics.get_height() {
            let r = rng_buffer.next() as u8;
            let g = rng_buffer.next() as u8;
            let b = rng_buffer.next() as u8;
            graphics.draw_pixel(x, y, Color::from_rgb(r, g, b));
        }
    }
}
