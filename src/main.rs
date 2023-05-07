mod graphics_window;
mod rng_buffer;
mod matrix_test;
mod world;

use std::ops::Add;
use std::time::{Duration, Instant};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::WindowId;
use crate::graphics_window::{Color, GraphicsBuffer, GraphicsWindow, WindowConfig};
use crate::rng_buffer::RngBuffer;
use crate::world::World;

struct FpsCounter {
    frame_time_buffer: [u128; 32],
    current_frame: usize,
    current_frame_start: Instant,
    mode: Mode
}

enum Mode {
    Off,
    Every4Frames,
    Every8Frames,
    Every16Frames,
    Every32Frames
}

impl FpsCounter {
    fn new() -> FpsCounter {
        FpsCounter::with_mode(Mode::Every16Frames)
    }

    fn with_mode(mode: Mode) -> FpsCounter {
        FpsCounter {
            frame_time_buffer: [0_u128; 32],
            current_frame: 0,
            current_frame_start: Instant::now(),
            mode,
        }
    }

    fn print_fps(&self) {
        let frame_count = match self.mode {
            Mode::Off => return,
            Mode::Every4Frames => 4,
            Mode::Every8Frames => 8,
            Mode::Every16Frames => 16,
            Mode::Every32Frames => 32,
        };
        if self.current_frame % frame_count != 0 { return; }

        let mut total_millis = 0;
        for i in 1..=frame_count {
            let frame = (self.current_frame + 32 - i) % 32;
            total_millis += self.frame_time_buffer[frame];
        }
        let average_frame_time_millis = total_millis / frame_count as u128;
        let fps = 1000_u128.checked_div(average_frame_time_millis).unwrap_or(0);
        println!("fps: {}", fps);
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn tick(&mut self) {
        self.frame_time_buffer[self.current_frame] = self.current_frame_start.elapsed().as_millis();
        self.current_frame_start = Instant::now();
        self.current_frame = (self.current_frame + 1) % 32;
        self.print_fps();
    }
}

fn main() {
    let mut fps_counter = FpsCounter::with_mode(Mode::Every32Frames);

    let mut world = World::new(256, 256);
    world.load();


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
                //let start = Instant::now();
                fps_counter.tick();


                world.step();
                world.draw(&mut graphics_window.get_graphics());
                //draw_noise(&mut graphics_window.get_graphics(), &mut rng_buffer);
                graphics_window.get_window().request_redraw();

                /*
                let elapsed = start.elapsed();
                frame_times[current_frame] = elapsed.as_millis();
                current_frame = (current_frame + 1) % FRAME_TIME_BUFFER_LENGTH;
                if current_frame == 0 {
                    let average_frame_time_millis = frame_times.iter().sum::<u128>() / FRAME_TIME_BUFFER_LENGTH as u128;
                    let fps = 1000 / average_frame_time_millis;
                    println!("fps: {}", fps);
                }
                */

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

#[allow(unused)]
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
