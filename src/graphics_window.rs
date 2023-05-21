use softbuffer::GraphicsContext;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

static mut PIXEL_BUFFER: Vec<u32> = Vec::new();
static mut WIDTH: usize = 0;
static mut HEIGHT: usize = 0;

pub fn width() -> usize {
    unsafe { WIDTH }
}

pub fn height() -> usize {
    unsafe { HEIGHT }
}

pub struct WindowConfig {
    pub title: String,
    pub resizable: bool,
    pub width: u32,
    pub height: u32,
}

impl WindowConfig {
    fn apply(&self, window: Window) -> Window {
        window.set_title(self.title.as_str());
        window.set_resizable(self.resizable);
        window.set_inner_size(PhysicalSize::new(self.width, self.height));
        return window;
    }
}

pub struct GraphicsWindow {
    window: Window,
    graphics_context: GraphicsContext,
}

fn build_window(event_loop: &EventLoop<()>, config: &WindowConfig) -> Window {
    config.apply(
        WindowBuilder::new()
            .build(&event_loop)
            .expect("should be able to build a window")
    )
}

pub fn build_graphics_window(config: WindowConfig) -> (GraphicsWindow, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = build_window(&event_loop, &config);
    let buffer_size = config.width * config.height;
    unsafe {
        PIXEL_BUFFER = vec![0; buffer_size as usize];
        WIDTH = config.width as usize;
        HEIGHT = config.height as usize;
    }
    let graphics_window = GraphicsWindow {
        graphics_context: unsafe { GraphicsContext::new(&window, &window) }.unwrap(),
        window,
    };
    return (graphics_window, event_loop);
}

impl GraphicsWindow {
    getter_ref!(window: Window);

    pub fn redraw(&mut self) {
        redraw(&mut self.graphics_context);
    }
}

pub fn clear(color: Color) {

    // TODO - Test a few solutions to find a reasonably performant one.
    //self.pixel_buffer.iter_mut().for_each(|px| *px = 0);

    unsafe {
        PIXEL_BUFFER.set_all(color.0);
    }

    //self.pixel_buffer[index as usize] = color.to_u32();
}

pub fn draw_pixel(x: usize, y: usize, color: Color) {
    let index = x + width() * y;
    unsafe {
        PIXEL_BUFFER[index] = color.0;
    }
}

fn redraw(graphics_context: &mut GraphicsContext) {
    graphics_context.set_buffer(unsafe { &PIXEL_BUFFER }, width() as u16, height() as u16);
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

pub struct Color(u32);

impl Color {
    #[allow(unused)]
    pub const BLACK: Color = Color::new(0, 0, 0);
    #[allow(unused)]
    pub const WHITE: Color = Color::new(255, 255, 255);
    #[allow(unused)]
    pub const RED: Color = Color::new(255, 0, 0);
    #[allow(unused)]
    pub const GREEN: Color = Color::new(0, 255, 0);
    #[allow(unused)]
    pub const BLUE: Color = Color::new(0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color(rgb_to_u32(r, g, b))
    }
}

const fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    b | (g << 8) | (r << 16)
}
