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

    pub frequency_timer: i32,
    pub duty_position: u8,
    pub length_timer_enabled: bool,
    pub length_timer: u8,
    pub envelope_volume: u8,
    pub envelope_timer: u8,
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
    channel1: Channel2,
    channel2: Channel2,
}

impl APU {
    pub fn new(sink: HeapProd<f32>, sample_rate: f32) -> APU {
        let mut regs = [0x0; 0x30];
        for &(addr, val) in AUDIO_INIT {
            regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val;
        }

        let wave = [0x0; 0x10];

        let channel1 = Channel2 {
            enabled: true,
            length_timer_enabled: false,
            frequency_timer: 0,
            duty_position: 0,
            length_timer: 0,
            envelope_volume: 1,
            envelope_timer: 0,
        };

        let channel2 = Channel2 {
            enabled: true,
            length_timer_enabled: false,
            frequency_timer: 0,
            duty_position: 0,
            length_timer: 0,
            envelope_volume: 1,
            envelope_timer: 0,
        };

        return APU {
            master_enable: true,
            regs: regs,
            wave: wave,
            sink: sink,
            current_cycles: 0.0,
            sample_rate: sample_rate,

            channel1: channel1,
            channel2: channel2,

            // frame sequencer
            frame_seq_state: FrameSequencer::Step0,
            frame_seq_cycles: 0,
        };
    }

    pub fn update(&mut self, instruction_cycles: u32) {
        let cycles_per_sample: f32 = CLOCK_SPEED as f32 / self.sample_rate;
        self.current_cycles += instruction_cycles as f32;
        self.update_channel1(instruction_cycles);
        self.update_channel2(instruction_cycles);

        // Frame Sequencer
        self.frame_seq_cycles += instruction_cycles;

        if self.frame_seq_cycles >= 8192 {
            match self.frame_seq_state {
                FrameSequencer::Step0 => {
                    self.clock_length_timers();
                    self.frame_seq_state = FrameSequencer::Step1;
                }
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

        while self.current_cycles >= cycles_per_sample {
            self.current_cycles -= cycles_per_sample;
            if self.master_enable {
                let channel1_output = self.play_channel1();
                let channel2_output = self.play_channel2();
                let _ = self.sink.try_push((channel1_output + channel2_output) / 2.0);
            }
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
            APU_RAM::NR52 => self.master_enable = val & 0b10000000 != 0,
            APU_RAM::NR51 => (), // Panning
            APU_RAM::NR50 => (), // Master Volume

            APU_RAM::NR14 => {
                if val & 0b10000000 != 0 {
                    self.channel1.enabled = true;
                    if self.channel1.length_timer == 0 {
                        self.channel1.length_timer = 64;
                    }
                    let period: i32 = (((self.read_register(APU_RAM::NR14)) as i32) & 7) << 8
                        | (self.read_register(APU_RAM::NR13) as i32);
                    self.channel1.frequency_timer = (2048 - period as i32) * 4;
                    self.channel1.envelope_volume =
                        (0b11110000 & self.read_register(APU_RAM::NR12)) >> 4;
                    self.channel1.envelope_timer = 0b111 & self.read_register(APU_RAM::NR12);

                    if self.channel1.length_timer == 0 {
                        self.channel1.length_timer = 64;
                    }
                }

                self.regs[addr as usize - 0xFF10] = val
            }

            APU_RAM::NR24 => {
                if val & 0b10000000 != 0 {
                    self.channel2.enabled = true;
                    if self.channel2.length_timer == 0 {
                        self.channel2.length_timer = 64;
                    }
                    let period: i32 = (((self.read_register(APU_RAM::NR24)) as i32) & 7) << 8
                        | (self.read_register(APU_RAM::NR23) as i32);
                    self.channel2.frequency_timer = (2048 - period as i32) * 4;
                    self.channel2.envelope_volume =
                        (0b11110000 & self.read_register(APU_RAM::NR22)) >> 4;
                    self.channel2.envelope_timer = 0b111 & self.read_register(APU_RAM::NR22);

                    if self.channel2.length_timer == 0 {
                        self.channel2.length_timer = 64;
                    }
                }

                self.regs[addr as usize - 0xFF10] = val
            }

            APU_RAM::AUDIO_RAM_START..=APU_RAM::AUDIO_RAM_END => {
                self.regs[addr as usize - 0xFF10] = val
            }
            APU_RAM::WAVE_RAM_START..=APU_RAM::WAVE_RAM_END => {
                self.wave[addr as usize - 0xFF30] = val
            }
            _ => (),
        }
    }

    pub fn update_channel1(&mut self, instruction_cycles: u32) {
        self.channel1.frequency_timer -= instruction_cycles as i32;
        while self.channel1.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR14)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR13) as i32);
            self.channel1.frequency_timer += (2048 - period as i32) * 4;
            self.channel1.duty_position = (self.channel1.duty_position + 1) % 8;
        }
    }

    pub fn update_channel2(&mut self, instruction_cycles: u32) {
        self.channel2.frequency_timer -= instruction_cycles as i32;
        while self.channel2.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR24)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR23) as i32);
            self.channel2.frequency_timer += (2048 - period as i32) * 4;
            self.channel2.duty_position = (self.channel2.duty_position + 1) % 8;
        }
    }

    pub fn play_channel1(&self) -> f32 {
        let duty_select = self.read_register(APU_RAM::NR11) >> 6 & 3;
        let pattern = WAVE_PATTERN_DUTY[duty_select as usize];
        let bit = (pattern >> self.channel1.duty_position) & 1;
        // 0 to 15
        let digital =
            if self.channel1.enabled && bit == 1 { self.channel1.envelope_volume } else { 0 };
        let analog = (digital as f32 / 7.5) - 1.0; // range: -1 to 1
        return analog;
    }

    pub fn play_channel2(&self) -> f32 {
        let duty_select = self.read_register(APU_RAM::NR21) >> 6 & 3;
        let pattern = WAVE_PATTERN_DUTY[duty_select as usize];
        let bit = (pattern >> self.channel2.duty_position) & 1;
        // 0 to 15
        let digital =
            if self.channel2.enabled && bit == 1 { self.channel2.envelope_volume } else { 0 };
        let analog = (digital as f32 / 7.5) - 1.0; // range: -1 to 1
        return analog;
    }

    pub fn clock_length_timers(&mut self) {
        if self.channel1.length_timer_enabled && self.read_register(APU_RAM::NR12) & 0b1000000 != 0
        {
            self.channel1.length_timer -= 1;
            if self.channel1.length_timer == 0 {
                self.channel1.length_timer_enabled = false
            }
        }
        if self.channel2.length_timer_enabled && self.read_register(APU_RAM::NR22) & 0b1000000 != 0
        {
            self.channel2.length_timer -= 1;
            if self.channel2.length_timer == 0 {
                self.channel2.length_timer_enabled = false
            }
        }
        // Fill in other channels
    }
}
