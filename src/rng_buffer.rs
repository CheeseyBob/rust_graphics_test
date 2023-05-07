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
            buffer: create_buffer(capacity),
            next: 0,
        }
    }

    /// Return the next value in the RNG buffer. The values are between 0.0 and 1.0.
    pub fn next(&mut self) -> f64 {
        self.next = (self.next + 1) % self.capacity;
        self.buffer[self.next]
    }
}
