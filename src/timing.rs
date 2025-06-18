use crate::consts::{CLOCK_SPEED, CYCLES_PER_FRAME};

pub struct Timing {
    pub current_cycles: u32,
    pub frame_cycles: u32,
}

impl Timing {
    pub fn new() -> Self {
        Self {
            current_cycles: 0,
            frame_cycles: 0,
        }
    }

    pub fn add_cycles(&mut self, cycles: u32) {
        self.current_cycles += cycles;
        self.frame_cycles += cycles;
    }

    pub fn is_frame_complete(&self) -> bool {
        self.frame_cycles >= CYCLES_PER_FRAME
    }

    pub fn reset_frame_cycles(&mut self) {
        self.frame_cycles = 0;
    }

    pub fn get_elapsed_time_us(&self) -> f32 {
        (self.current_cycles as f32 / CLOCK_SPEED as f32) * 1_000_000.0
    }
} 