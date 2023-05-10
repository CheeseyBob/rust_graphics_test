use std::time::Instant;

pub struct FpsCounter {
    frame_time_buffer: [u128; 32],
    current_frame: usize,
    current_frame_start: Instant,
    mode: Mode
}

pub enum Mode {
    Off,
    Every4Frames,
    Every8Frames,
    Every16Frames,
    Every32Frames
}

impl FpsCounter {
    #[allow(unused)]
    pub fn every_4_frames() -> FpsCounter {
        Self::with_mode(Mode::Every4Frames)
    }

    #[allow(unused)]
    pub fn every_8_frames() -> FpsCounter {
        Self::with_mode(Mode::Every8Frames)
    }

    #[allow(unused)]
    pub fn every_16_frames() -> FpsCounter {
        Self::with_mode(Mode::Every16Frames)
    }

    #[allow(unused)]
    pub fn every_32_frames() -> FpsCounter {
        Self::with_mode(Mode::Every32Frames)
    }

    #[allow(unused)]
    pub fn new() -> FpsCounter {
        FpsCounter::with_mode(Mode::Off)
    }

    #[allow(unused)]
    pub fn with_mode(mode: Mode) -> FpsCounter {
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
        println!("fps: {} (frame-time: {} ms)", fps, average_frame_time_millis);
    }

    #[allow(unused)]
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn tick(&mut self) {
        self.frame_time_buffer[self.current_frame] = self.current_frame_start.elapsed().as_millis();
        self.current_frame_start = Instant::now();
        self.current_frame = (self.current_frame + 1) % 32;
        self.print_fps();
    }
}