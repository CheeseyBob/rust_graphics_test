use softbuffer::GraphicsContext;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct GraphicsWindow {
    pub window: Window,
    graphics_context: GraphicsContext,
    pub graphics_buffer: GraphicsBuffer,
}

impl GraphicsWindow {
    pub fn build(event_loop: &EventLoop<()>) -> GraphicsWindow {
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        GraphicsWindow {
            graphics_context: unsafe { GraphicsContext::new(&window, &window) }.unwrap(),
            graphics_buffer: GraphicsBuffer::create_for_window(&window),
            window,
        }
    }

    pub fn redraw(&mut self) {
        self.graphics_buffer.redraw(&mut self.graphics_context);
    }
}

pub struct GraphicsBuffer {
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

    pub fn clear(&mut self, color: Color) {

        // TODO ...
        //self.pixel_buffer.iter_mut().for_each(|px| *px = 0);

        self.pixel_buffer.set_all(color.to_u32());

        //self.pixel_buffer[index as usize] = color.to_u32();
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = x + self.width * y;
        self.pixel_buffer[index as usize] = color.to_u32();
    }

    pub fn get_height(&self) -> u32 { self.height }

    pub fn get_width(&self) -> u32 { self.width }

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

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    #[allow(unused)]
    pub const BLACK: Color = Color::from_rgb(0, 0, 0);
    #[allow(unused)]
    pub const WHITE: Color = Color::from_rgb(255, 255, 255);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    fn to_u32(&self) -> u32 {
        let (red, green, blue) = (self.r as u32, self.g as u32, self.b as u32);
        blue | (green << 8) | (red << 16)
    }
}