use crate::consts::{APU_RAM, AUDIO_INIT, CLOCK_SPEED};
use ringbuf::{traits::Observer, traits::Producer, HeapProd};

pub enum FrameSequencer {
    Step0, // Length Counter
    Step1, // None
    Step2, // Length Counter, Sweep
    Step3, // None
    Step4, // Length Counter
    Step5, // None
    Step6, // Length Counter, Sweep
    Step7, // Volume Envelope
}

pub struct Channel2 {
    pub enabled: bool,

    pub frequency_timer: i32, // NR23/24
    pub duty_position: u8,
    pub length_cycle: u8,
    pub volume_envelope: u8,

    pub initial_volume: u8,
    pub period_low: u8,
}

pub const WAVE_PATTERN_DUTY: [u8; 4] = [
    0b00000001, // 12.5
    0b00000011, // 25
    0b00001111, // 50
    0b11111100, // 75
];

pub struct APU {
    master_enable: bool,
    regs: [u8; 0x30],
    wave: [u8; 0x10],

    sink: HeapProd<f32>,
    sample_rate: f32,
    current_cycles: f32, // fractional T-cycle counter

    frame_seq_state: FrameSequencer,
    frame_seq_cycles: u32,
    // phase: f32,
    channel2: Channel2,
}

impl APU {
    pub fn new(sink: HeapProd<f32>, sample_rate: f32) -> APU {
        let mut regs = [0x0; 0x30];
        for &(addr, val) in AUDIO_INIT {
            regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val;
        }

        let channel2 = Channel2 {
            enabled: true,
            frequency_timer: 0,
            duty_position: 0,
            length_cycle: 0,
            volume_envelope: 0,

            initial_volume: 0,
            period_low: 0,
        };

        let wave = [0x0; 0x10];
        return APU {
            master_enable: true,
            regs: regs,
            wave: wave,
            sink: sink,
            current_cycles: 0.0,
            sample_rate: sample_rate,

            // frame sequencer
            frame_seq_state: FrameSequencer::Step0,
            frame_seq_cycles: 0,
            channel2: channel2,
        };
    }

    pub fn update(&mut self, instruction_cycles: u32) {
        let cycles_per_sample: f32 = CLOCK_SPEED as f32 / self.sample_rate;
        self.current_cycles += instruction_cycles as f32;
        self.clock_channel2(instruction_cycles);

        // Frame Sequencer
        self.frame_seq_cycles += instruction_cycles;

        if self.frame_seq_cycles >= 8192 {
            match self.frame_seq_state {
                FrameSequencer::Step0 => self.frame_seq_state = FrameSequencer::Step1,
                FrameSequencer::Step1 => self.frame_seq_state = FrameSequencer::Step2,
                FrameSequencer::Step2 => self.frame_seq_state = FrameSequencer::Step3,
                FrameSequencer::Step3 => self.frame_seq_state = FrameSequencer::Step4,
                FrameSequencer::Step4 => self.frame_seq_state = FrameSequencer::Step5,
                FrameSequencer::Step5 => self.frame_seq_state = FrameSequencer::Step6,
                FrameSequencer::Step6 => self.frame_seq_state = FrameSequencer::Step7,
                FrameSequencer::Step7 => self.frame_seq_state = FrameSequencer::Step0,
            }
            self.frame_seq_cycles -= 8192;
        }
        // Frequency Timer

        while self.current_cycles >= cycles_per_sample {
            self.current_cycles -= cycles_per_sample;

            let duty_select = self.read_register(APU_RAM::NR21 >> 6) & 3;
            let pattern = WAVE_PATTERN_DUTY[duty_select as usize];
            let bit = (pattern >> self.channel2.duty_position) & 1;
            let volume = (self.read_register(APU_RAM::NR22) >> 4) & 0xF;
            let digital = if self.channel2.enabled && bit == 1 { volume } else { 0 };
            let analog = (digital as f32 / 7.5) - 1.0; // range: -1 to 1
            let prime = 2; // a couple of LR frames' worth of mono samples
                           // for _ in 0..prime {
                           //
            let cap = self.sink.capacity().get(); // total slots
            let filled = self.sink.occupied_len(); // samples waiting to be consumed
            let pct = (filled as f32 / cap as f32) * 100.0;
            let _ = self.sink.try_push(analog);
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            APU_RAM::AUDIO_RAM_START..=APU_RAM::AUDIO_RAM_END => self.regs[addr as usize - 0xFF10],
            APU_RAM::WAVE_RAM_START..=APU_RAM::WAVE_RAM_END => self.wave[addr as usize - 0xFF30],
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            // AUDIO_RAM_NR52 => self.master_enable = val & 0x07 == 1,
            APU_RAM::AUDIO_RAM_START..APU_RAM::AUDIO_RAM_END => {
                self.regs[addr as usize - 0xFF10] = val
            }
            APU_RAM::WAVE_RAM_START..=APU_RAM::WAVE_RAM_END => {
                self.wave[addr as usize - 0xFF30] = val
            }
            _ => (),
        }
    }

    pub fn clock_channel2(&mut self, instruction_cycles: u32) {
        self.channel2.frequency_timer -= instruction_cycles as i32;
        while self.channel2.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR24)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR23) as i32);
            self.channel2.frequency_timer += (2048 - period as i32) * 4;
            self.channel2.duty_position = (self.channel2.duty_position + 1) % 8;
        }
    }
}
