use crate::consts::{
    AUDIO_INIT, AUDIO_RAM_END, AUDIO_RAM_START, DMG0_IO_INIT, NR10, WAVE_RAM_END, WAVE_RAM_START,
};
use rodio::source::{SineWave, Source, SquareWave};
use rodio::stream::DeviceSinkBuilder;
use std::time::Duration;

pub struct Channel1 {
    pub enabled: bool,
}
//
pub struct Channel2 {
    pub enabled: bool,
    pub length_timer: u8,
    pub duty_cycle: u8,

    pub initial_volume: u8,
    pub env_dir: u8,
    pub sweep_pace: u8,

    pub period_low: u8,
}

pub struct Channel3 {}

pub struct APU {
    master_enable: bool,
    regs: [u8; 0x30],
    wave: [u8; 0x10],
}

impl APU {
    pub fn new() -> APU {
        let mut regs = [0x0; 0x30];
        for &(addr, val) in AUDIO_INIT {
            regs[addr as usize - AUDIO_RAM_START as usize] = val;
        }

        let wave = [0x0; 0x10];
        return APU { master_enable: true, regs: regs, wave: wave };
    }

    pub fn update(&self, instruction_cycles: u32) {
        // let device_sink = DeviceSinkBuilder::open_default_sink().expect("Error: can't open sink");
        // let source = SquareWave::new(440.0).take_duration(Duration::from_secs(5));
        // device_sink.mixer().add(source);
        // std::thread::sleep(Duration::from_secs(5));
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            AUDIO_RAM_START..=AUDIO_RAM_END => self.regs[addr as usize - 0xFF10],
            WAVE_RAM_START..=WAVE_RAM_END => self.wave[addr as usize - 0xFF30],
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            AUDIO_RAM_START..=AUDIO_RAM_END => self.regs[addr as usize - 0xFF10] = val,
            WAVE_RAM_START..=WAVE_RAM_END => self.wave[addr as usize - 0xFF30] = val,
            _ => (),
        }
    }
}
