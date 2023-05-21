/// Creates a public getter function for given (Copy) field.
#[allow(unused)]
macro_rules! getter {
    ($var:ident: $t:ty) => {
        pub fn $var(&self) -> $t { self.$var }
    };
}

/// Creates a public reference-getter function for given field.
#[allow(unused)]
macro_rules! getter_mut {
    ($var:ident: $typ:ty) => {
        pub fn $var(&mut self) -> &mut $typ { &mut self.$var }
    };
}

/// Creates a public reference-getter function for given field.
#[allow(unused)]
macro_rules! getter_ref {
    ($var:ident: $typ:ty) => {
        pub fn $var(&self) -> &$typ { &self.$var }
    };
}

mod graphics_window;
mod rng_buffer;
mod matrix_test;
mod world;
mod fps_counter;
mod world_processor;
mod entity;
mod action;

use std::ops::Add;
use std::time::{Duration, Instant};
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow};
use winit::window::WindowId;
use crate::fps_counter::FpsCounter;
use crate::graphics_window::WindowConfig;
use crate::world::World;


fn main() {

    /*
    mutex_vec_alt::test();
    std::process::exit(0);
    */

    /*
    match matrix_test::run() {
        None => exit(0),
        Some(_) => {},
    }
    */

    /**********************************************************************************************/

    let draw_is_enabled = true;
    let (width, height) = (1800, 900);
    let window_config: WindowConfig = WindowConfig {
        title: String::from("Test"),
        resizable: false,
        width: width as u32,
        height: height as u32,
    };
    let target_fps = 1000;
    let target_frame_time: Duration = Duration::from_millis(1000 / target_fps);
    let mut next_tick = Instant::now().add(target_frame_time);

    let mut fps_counter = FpsCounter::every_32_frames();

    rng_buffer::init();

    let mut world = World::new(width, height);
    load_test_world(&mut world, 50_000);
    world_processor::init(world);

    let (mut graphics_window, event_loop) = graphics_window::build_graphics_window(window_config);

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
                world_processor::step();
                if draw_is_enabled {
                    world_processor::draw();
                }
                graphics_window.window().request_redraw();
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

fn load_test_world(world: &mut World, entity_count: u32) {
    let mut count = 0;
    while count < entity_count {
        let x = (rng_buffer::generate_next() * world.width() as f64) as usize;
        let y = (rng_buffer::next() * world.height() as f64) as usize;
        let entity = entity::Entity::new(x, y, &world);
        if world.place_entity(entity).is_ok() {
            count += 1;
        }
    }
}