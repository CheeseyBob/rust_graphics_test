use rand::random;

const BUFFER_SIZE_U16: usize = 65536;
static mut BUFFER_U16: [f64; BUFFER_SIZE_U16] = [0.0; BUFFER_SIZE_U16];
static mut NEXT: u16 = 0;

pub fn init() {
    unsafe {
        for i in 0..BUFFER_SIZE_U16 {
            BUFFER_U16[i] = random();
        }
    }
}

pub fn generate_next() -> f64 {
    increment();
    regenerate();
    read()
}

pub fn next() -> f64 {
    increment();
    read()
}

fn increment() {
    // Safety: TODO - Needs testing.
    unsafe {
        NEXT = NEXT.wrapping_add(1);
    }
}

fn read() -> f64 {
    // Safety: We don't care about data races here, as we are reading random values.
    unsafe {
        BUFFER_U16[NEXT as usize]
    }
}

fn regenerate() {
    // Safety: We don't care about data races here, as we are writing random values.
    unsafe {
        BUFFER_U16[NEXT as usize] = random();
    }
}
