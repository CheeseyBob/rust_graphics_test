mod rng_buffer;

use std::ops::{Add};
use std::time::{Duration, Instant};
use softbuffer::GraphicsContext;
use winit::dpi::{PhysicalSize};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder, WindowId};
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


    for x in 0..buffer.width {
        for y in 0..buffer.height {
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

struct GraphicsWindow {
    window: Window,
    graphics_context: GraphicsContext,
    graphics_buffer: GraphicsBuffer,
}

impl GraphicsWindow {
    fn build(event_loop: &EventLoop<()>) -> GraphicsWindow {
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        GraphicsWindow {
            graphics_context: unsafe { GraphicsContext::new(&window, &window) }.unwrap(),
            graphics_buffer: GraphicsBuffer::create_for_window(&window),
            window,
        }
    }

    fn redraw(&mut self) {
        self.graphics_buffer.redraw(&mut self.graphics_context);
    }
}

struct GraphicsBuffer {
    pixel_buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl GraphicsBuffer {
    fn create_for_window(window: &Window) -> Self {
        let window_size = window.inner_size();
        let buffer_size = window_size.width * window_size.height;
        Self {
            pixel_buffer: vec![0; buffer_size as usize],
            width: window_size.width,
            height: window_size.height,
        }
    }

    fn clear(&mut self, color: Color) {

        // TODO ...
        //self.pixel_buffer.iter_mut().for_each(|px| *px = 0);

        self.pixel_buffer.set_all(color.to_u32());

        //self.pixel_buffer[index as usize] = color.to_u32();
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = x + self.width * y;
        self.pixel_buffer[index as usize] = color.to_u32();
    }

    fn redraw(&self, graphics_context: &mut GraphicsContext) {
        graphics_context.set_buffer(&self.pixel_buffer, self.width as u16, self.height as u16);
    }
}

/*************************************************************/
// Allows calling .set_all(some_value) on an array to set all values in the array to some_value.
// Taken from https://stackoverflow.com/a/49193323/17816986
trait SetAll {
    type Elem;
    fn set_all(&mut self, value: Self::Elem);
}

impl<T> SetAll for [T] where T: Clone {
    type Elem = T;
    fn set_all(&mut self, value: Self::Elem) {
        for e in self {
            *e = value.clone();
        }
    }
}
/*************************************************************/

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    #[allow(unused)]
    const BLACK: Color = Color::from_rgb(0, 0, 0);
    #[allow(unused)]
    const WHITE: Color = Color::from_rgb(255, 255, 255);

    const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    fn to_u32(&self) -> u32 {
        let (red, green, blue) = (self.r as u32, self.g as u32, self.b as u32);
        blue | (green << 8) | (red << 16)
    }
}

#[allow(unused)]
fn test_main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //event_loop.run(event_handler);

    dbg!(window.is_visible());
}

#[allow(unused)]
fn example_main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                let buffer = (0..((width * height) as usize))
                    .map(|index| {
                        let y = index / (width as usize);
                        let x = index % (width as usize);
                        let red = x % 255;
                        let green = y % 255;
                        let blue = (x * y) % 255;

                        let color = blue | (green << 8) | (red << 16);

                        color as u32
                    })
                    .collect::<Vec<_>>();

                graphics_context.set_buffer(&buffer, width as u16, height as u16);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}