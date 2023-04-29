use rand::Rng;

pub struct RngBuffer {
    buffer: Vec<i32>, // TODO - Change to a generic type.
    capacity: usize,
    next: usize,
}

impl RngBuffer {
    pub fn new(capacity: usize) -> RngBuffer {
        RngBuffer {
            capacity,
            buffer: vec![0; capacity],
            next: 0,
        }
    }

    pub fn init_new(capacity: usize, _generator: ()) -> RngBuffer {
        let mut rng_buffer = RngBuffer::new(capacity);
        rng_buffer.init(_generator);
        return rng_buffer;
    }

    pub fn init(&mut self, _generator: ()) { // TODO - Needs the generating function to be supplied too.
        for i in 0..self.capacity {
            self.buffer[i] = rand::thread_rng().gen_range(0..255);
        }
    }

    pub fn next(&mut self) -> i32 {
        self.next = (self.next + 1) % self.capacity;
        self.buffer[self.next]
    }
}
