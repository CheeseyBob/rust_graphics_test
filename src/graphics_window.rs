use softbuffer::GraphicsContext;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

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

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct GraphicsWindow {
    window: Window,
    graphics_context: GraphicsContext,
    graphics_buffer: GraphicsBuffer,
}

fn build_window(event_loop: &EventLoop<()>, config: &WindowConfig) -> Window {
    config.apply(
        WindowBuilder::new()
            .build(&event_loop)
            .expect("should be able to build a window")
    )
}

impl GraphicsWindow {
    getter_ref!(window: Window);
    getter_mut!(graphics_buffer: GraphicsBuffer);

    pub fn build(config: WindowConfig) -> (GraphicsWindow, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = build_window(&event_loop, &config);
        let graphics_window = GraphicsWindow {
            graphics_context: unsafe { GraphicsContext::new(&window, &window) }.unwrap(),
            graphics_buffer: GraphicsBuffer::new(&config),
            window,
        };
        return (graphics_window, event_loop);
    }

    pub fn redraw(&mut self) {
        self.graphics_buffer.redraw(&mut self.graphics_context);
    }
}

pub struct GraphicsBuffer {
    pixel_buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl GraphicsBuffer {
    getter!(height: usize);
    getter!(width: usize);

    fn new(config: &WindowConfig) -> Self {
        let buffer_size = config.width * config.height;
        Self {
            pixel_buffer: vec![0; buffer_size as usize],
            width: config.width as usize,
            height: config.height as usize,
        }
    }

    pub fn clear(&mut self, color: Color) {

        // TODO - Test a few solutions to find a reasonably performant one.
        //self.pixel_buffer.iter_mut().for_each(|px| *px = 0);

        self.pixel_buffer.set_all(color.0);

        //self.pixel_buffer[index as usize] = color.to_u32();
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = x + self.width * y;
        self.pixel_buffer[index] = color.0;
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

pub struct Color(u32);

impl Color {
    #[allow(unused)]
    pub const BLACK: Color = Color::new(0, 0, 0);
    #[allow(unused)]
    pub const WHITE: Color = Color::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color(rgb_to_u32(r, g, b))
    }
}

const fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    b | (g << 8) | (r << 16)
}
