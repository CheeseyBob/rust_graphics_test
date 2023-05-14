use std::sync::Arc;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::random;

pub struct RngBuffer {
    buffer: Vec<f64>,
    capacity: usize,
    next: usize,
}

fn create_buffer(capacity: usize) -> Vec<f64> {
    let mut buffer = Vec::with_capacity(capacity);
    for _ in 0..capacity {
        buffer.push(random());
    }
    return buffer;
}

impl RngBuffer {
    pub fn with_capacity(capacity: usize) -> RngBuffer {
        RngBuffer {
            capacity,
            buffer: create_buffer(capacity), // TODO - Test if this works: vec![random(); capacity],
            next: 0,
        }
    }

    /// Return the next value in the RNG buffer. The values are between 0.0 and 1.0.
    pub fn next(&mut self) -> f64 {
        self.next = (self.next + 1) % self.capacity;
        self.buffer[self.next]
    }

    /// Regenerate and return the next value in te RNG buffer.
    /// Use this instead of `next()` only when you want to refresh the values in the buffer.
    pub fn generate_next(&mut self) -> f64 {
        self.next = (self.next + 1) % self.capacity;
        self.buffer[self.next] = random();
        self.buffer[self.next]
    }
}

static DEFAULT: Lazy<Mutex<RngBuffer>> = Lazy::new(|| {
    Mutex::new(RngBuffer::with_capacity(10_000))
});

pub fn generate_next() -> f64 {
    DEFAULT.lock().generate_next()
}

pub fn next() -> f64 {
    DEFAULT.lock().next()
}
