use crate::consts::{APU_RAM, AUDIO_INIT, CLOCK_SPEED};
use ringbuf::{traits::Producer, HeapProd};

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

pub struct Envelope {
    pub volume: u8,
    pub timer: u8,
}

impl Envelope {
    pub fn new(volume: u8) -> Self {
        Envelope { volume, timer: 0 }
    }

    pub fn clock(&mut self, control_reg: u8) {
        let envelope_sweep_pace = control_reg & 0b111;
        if envelope_sweep_pace != 0 {
            self.timer = self.timer.saturating_sub(1);
            if self.timer == 0 {
                self.timer = envelope_sweep_pace;
                if (control_reg & 0b1000) != 0 {
                    self.volume = self.volume.saturating_add(1).min(15);
                } else {
                    self.volume = self.volume.saturating_sub(1);
                }
            }
        }
    }
}

pub struct Length {
    pub timer: u16,
}

impl Length {
    pub fn new() -> Self {
        Length { timer: 0 }
    }

    pub fn clock(&mut self, control_reg: u8) -> bool {
        let enabled = if self.timer != 0 && (control_reg & 0b1000000) != 0 {
            self.timer -= 1;
            self.timer == 0
        } else {
            false
        };

        return enabled;
    }
}

pub struct Channel1 {
    pub enabled: bool,

    pub frequency_timer: i32,
    pub duty_position: u8,
    pub length: Length,
    pub envelope: Envelope,

    pub sweep_frequency: i32,
    pub sweep_timer: u8,
    pub sweep_enabled: bool,
}

pub struct Channel2 {
    pub enabled: bool,

    pub frequency_timer: i32,
    pub duty_position: u8,
    pub length: Length,
    pub envelope: Envelope,
}

pub struct Channel3 {
    pub enabled: bool,

    pub frequency_timer: i32,
    pub wave_position: u8,
    pub length: Length,
}

pub struct Channel4 {
    pub enabled: bool,
    pub frequency_timer: i32,
    pub length: Length,
    pub envelope: Envelope,
    pub lfsr: u16,
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
    wave_ram: [u8; 0x10], // 4 bits is 1 sample

    sink: HeapProd<f32>,
    sample_rate: f32,
    current_cycles: f32, // fractional T-cycle counter

    frame_sequence_state: FrameSequencer,
    frame_sequence_cycles: u32,
    // phase: f32,
    channel1: Channel1,
    channel2: Channel2,
    channel3: Channel3,
    channel4: Channel4,
}

impl APU {
    pub fn new(sink: HeapProd<f32>, sample_rate: f32) -> APU {
        let mut regs = [0x0; 0x30];
        for &(addr, val) in AUDIO_INIT {
            regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val;
        }

        let wave = [0x0; 0x10];

        let channel1 = Channel1 {
            enabled: true,
            frequency_timer: 0,
            duty_position: 0,
            length: Length::new(),
            envelope: Envelope::new(1),

            sweep_frequency: 0,
            sweep_timer: 8,
            sweep_enabled: true,
        };

        let channel2 = Channel2 {
            enabled: true,
            frequency_timer: 0,
            duty_position: 0,
            length: Length::new(),
            envelope: Envelope::new(1),
        };

        let channel3 =
            Channel3 { enabled: true, frequency_timer: 0, wave_position: 0, length: Length::new() };

        let channel4 = Channel4 {
            enabled: true,
            frequency_timer: 0,
            length: Length::new(),
            envelope: Envelope::new(1),
            lfsr: 0,
        };

        return APU {
            master_enable: true,
            regs: regs,
            wave_ram: wave,
            sink: sink,
            current_cycles: 0.0,
            sample_rate: sample_rate,

            channel1: channel1,
            channel2: channel2,
            channel3: channel3,
            channel4: channel4,

            // frame sequencer
            frame_sequence_state: FrameSequencer::Step0,
            frame_sequence_cycles: 0,
        };
    }

    pub fn update(&mut self, instruction_cycles: u32) {
        let cycles_per_sample: f32 = CLOCK_SPEED as f32 / self.sample_rate;
        // println!("{}", cycles_per_sample);
        self.current_cycles += instruction_cycles as f32;
        self.clock_frequency_timers(instruction_cycles);

        // Frame Sequencer
        self.frame_sequence_cycles += instruction_cycles;

        if self.frame_sequence_cycles >= 8192 {
            match self.frame_sequence_state {
                FrameSequencer::Step0 => {
                    self.clock_length_timers();
                    self.frame_sequence_state = FrameSequencer::Step1;
                }
                FrameSequencer::Step1 => self.frame_sequence_state = FrameSequencer::Step2,
                FrameSequencer::Step2 => {
                    self.clock_length_timers();
                    self.clock_sweep();
                    self.frame_sequence_state = FrameSequencer::Step3;
                }
                FrameSequencer::Step3 => self.frame_sequence_state = FrameSequencer::Step4,
                FrameSequencer::Step4 => {
                    self.clock_length_timers();
                    self.frame_sequence_state = FrameSequencer::Step5
                }
                FrameSequencer::Step5 => {
                    self.frame_sequence_state = FrameSequencer::Step6;
                }
                FrameSequencer::Step6 => {
                    self.clock_length_timers();
                    self.clock_sweep();
                    self.frame_sequence_state = FrameSequencer::Step7;
                }
                FrameSequencer::Step7 => {
                    self.clock_envelope();
                    self.frame_sequence_state = FrameSequencer::Step0;
                }
            }
            self.frame_sequence_cycles -= 8192;
        }

        while self.current_cycles >= cycles_per_sample {
            self.current_cycles -= cycles_per_sample;
            if self.master_enable {
                let channel1_output = self.output_channel1();
                let channel2_output = self.output_channel2();
                let channel3_output = self.output_channel3();
                let channel4_output = self.output_channel4();

                // NR50: master volume, averaged across both sides for the mono
                // mix; each side's 0-7 value scales as (vol+1)/8. Games rely on
                // this for music fade-outs.
                let master_volume_reg = self.read_register(APU_RAM::NR50);
                let left_volume = ((master_volume_reg & 0b111) + 1) as f32;
                let right_volume = (((master_volume_reg >> 4) & 0b111) + 1) as f32;
                let volume = (left_volume + right_volume) / 16.0;

                let _ = self.sink.try_push(
                    (channel1_output + channel2_output + channel3_output + channel4_output)
                        * volume
                        / 20.0,
                );
            }
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            APU_RAM::NR52 => {
                let output = 0b01110000
                    | ((self.master_enable as u8) << 7)
                    | ((self.channel4.enabled as u8) << 3)
                    | ((self.channel3.enabled as u8) << 2)
                    | ((self.channel2.enabled as u8) << 1)
                    | ((self.channel1.enabled as u8) << 0);
                output
            }
            APU_RAM::AUDIO_RAM_START..=APU_RAM::AUDIO_RAM_END => {
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize]
            }
            APU_RAM::WAVE_RAM_START..=APU_RAM::WAVE_RAM_END => {
                self.wave_ram[addr as usize - APU_RAM::WAVE_RAM_START as usize]
            }
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            APU_RAM::NR52 => self.master_enable = val & 0b10000000 != 0,
            // NR50 (master volume) and NR51 (panning) fall through to the
            // generic store below: games read these back and modify them
            // (e.g. Pokemon's music fade-out decrements NR50 until it reads 0),
            // so dropping the write breaks read-modify-write loops.
            APU_RAM::NR11 => {
                self.channel1.length.timer = 64 - (val & 0b11_1111) as u16;
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR12 => {
                if val & 0b11111000 == 0 {
                    self.channel1.enabled = false;
                }
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR14 => {
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val;
                if val & 0b10000000 != 0 {
                    let dac_enabled = self.read_register(APU_RAM::NR12) & 0b11111000 != 0;
                    if dac_enabled {
                        self.channel1.enabled = true;
                    }
                    if self.channel1.length.timer == 0 {
                        self.channel1.length.timer = 64;
                    }
                    let period: i32 = (((self.read_register(APU_RAM::NR14)) as i32) & 7) << 8
                        | (self.read_register(APU_RAM::NR13) as i32);
                    self.channel1.frequency_timer = (2048 - period as i32) * 4;
                    self.channel1.envelope.volume =
                        (0b11110000 & self.read_register(APU_RAM::NR12)) >> 4;
                    self.channel1.envelope.timer = 0b111 & self.read_register(APU_RAM::NR12);

                    if self.channel1.length.timer == 0 {
                        self.channel1.length.timer = 64;
                    }

                    self.trigger_sweep();
                }
            }

            APU_RAM::NR21 => {
                self.channel2.length.timer = 64 - (val & 0b11_1111) as u16;
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR22 => {
                if val & 0b11111000 == 0 {
                    self.channel2.enabled = false;
                }
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR24 => {
                if val & 0b1000_0000 != 0 {
                    let dac_enabled = self.read_register(APU_RAM::NR22) & 0b11111000 != 0;
                    if dac_enabled {
                        self.channel2.enabled = true;
                    }
                    if self.channel2.length.timer == 0 {
                        self.channel2.length.timer = 64;
                    }
                    let period: i32 = (((self.read_register(APU_RAM::NR24)) as i32) & 7) << 8
                        | (self.read_register(APU_RAM::NR23) as i32);
                    self.channel2.frequency_timer = (2048 - period as i32) * 4;
                    self.channel2.envelope.volume =
                        (0b1111_0000 & self.read_register(APU_RAM::NR22)) >> 4;
                    self.channel2.envelope.timer = 0b111 & self.read_register(APU_RAM::NR22);
                }

                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR31 => {
                self.channel3.length.timer = 256 - val as u16;
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR34 => {
                let dac_enabled = (self.read_register(APU_RAM::NR30) & 0b10000000) != 0;
                if val & 0b1000_0000 != 0 {
                    if dac_enabled {
                        self.channel3.enabled = true;
                    }
                    if self.channel3.length.timer == 0 {
                        self.channel3.length.timer = 256;
                    }
                    let period: i32 = (((self.read_register(APU_RAM::NR34)) as i32) & 7) << 8
                        | (self.read_register(APU_RAM::NR33) as i32);
                    self.channel3.frequency_timer = (2048 - period as i32) * 2;
                    self.channel3.wave_position = 0;
                }

                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR41 => {
                self.channel4.length.timer = 64 - (val & 0b11_1111) as u16;
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR42 => {
                if val & 0b11111000 == 0 {
                    self.channel4.enabled = false;
                }
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::NR44 => {
                if val & 0b1000_0000 != 0 {
                    let dac_enabled = self.read_register(APU_RAM::NR42) & 0b11111000 != 0;
                    if dac_enabled {
                        self.channel4.enabled = true;
                    }
                    if self.channel4.length.timer == 0 {
                        self.channel4.length.timer = 64;
                    }
                    let clock_divider_bits = self.read_register(APU_RAM::NR43) & 0b111;
                    let divisor = match clock_divider_bits {
                        0 => 8,
                        1 => 16,
                        2 => 32,
                        3 => 48,
                        4 => 64,
                        5 => 80,
                        6 => 96,
                        7 => 112,
                        _ => 8,
                    };

                    let shift = (self.read_register(APU_RAM::NR43) & 0b11110000) >> 4;
                    self.channel4.frequency_timer = divisor << shift;
                    self.channel4.envelope.volume =
                        (0b1111_0000 & self.read_register(APU_RAM::NR42)) >> 4;
                    self.channel4.envelope.timer = 0b111 & self.read_register(APU_RAM::NR42);
                }

                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }

            APU_RAM::AUDIO_RAM_START..=APU_RAM::AUDIO_RAM_END => {
                self.regs[addr as usize - APU_RAM::AUDIO_RAM_START as usize] = val
            }
            APU_RAM::WAVE_RAM_START..=APU_RAM::WAVE_RAM_END => {
                self.wave_ram[addr as usize - APU_RAM::WAVE_RAM_START as usize] = val
            }
            _ => (),
        }
    }

    pub fn clock_frequency_timers(&mut self, instruction_cycles: u32) {
        self.channel1.frequency_timer -= instruction_cycles as i32;
        while self.channel1.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR14)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR13) as i32);
            self.channel1.frequency_timer += (2048 - period as i32) * 4;
            self.channel1.duty_position = (self.channel1.duty_position + 1) % 8;
        }

        self.channel2.frequency_timer -= instruction_cycles as i32;
        while self.channel2.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR24)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR23) as i32);
            self.channel2.frequency_timer += (2048 - period as i32) * 4;
            self.channel2.duty_position = (self.channel2.duty_position + 1) % 8;
        }

        self.channel3.frequency_timer -= instruction_cycles as i32;
        while self.channel3.frequency_timer <= 0 {
            let period: i32 = (((self.read_register(APU_RAM::NR34)) as i32) & 7) << 8
                | (self.read_register(APU_RAM::NR33) as i32);
            self.channel3.frequency_timer += (2048 - period as i32) * 2;
            self.channel3.wave_position = (self.channel3.wave_position + 1) % 32;
        }

        self.channel4.frequency_timer -= instruction_cycles as i32;
        while self.channel4.frequency_timer <= 0 {
            let clock_divider_bits = self.read_register(APU_RAM::NR43) & 0b111;
            let divisor = match clock_divider_bits {
                0 => 8,
                1 => 16,
                2 => 32,
                3 => 48,
                4 => 64,
                5 => 80,
                6 => 96,
                7 => 112,
                _ => 8,
            };

            let shift = (self.read_register(APU_RAM::NR43) & 0b11110000) >> 4;
            self.channel4.frequency_timer += divisor << shift;

            // Update LSFR
            let tap_bit = !(self.channel4.lfsr & 1 ^ ((self.channel4.lfsr >> 1) & 1)) & 1;
            self.channel4.lfsr = (tap_bit << 14) | self.channel4.lfsr >> 1;
            if (self.read_register(APU_RAM::NR43) & 0b1000) != 0 {
                self.channel4.lfsr = (!(1 << 6)) & self.channel4.lfsr;
                self.channel4.lfsr = (tap_bit << 6) | self.channel4.lfsr;
            }
        }
    }

    pub fn output_channel1(&self) -> f32 {
        let duty_select = self.read_register(APU_RAM::NR11) >> 6 & 3;
        let pattern = WAVE_PATTERN_DUTY[duty_select as usize];
        let bit = (pattern >> self.channel1.duty_position) & 1;
        // 0 to 15
        let digital =
            if self.channel1.enabled && bit == 1 { self.channel1.envelope.volume } else { 0 };

        // DAC
        let dac_enabled = (self.read_register(APU_RAM::NR12) & 0b11111000) != 0;
        let analog = if dac_enabled { (digital as f32 / 7.5) - 1.0 } else { 0.0 };
        return analog;
    }

    pub fn output_channel2(&self) -> f32 {
        let duty_select = self.read_register(APU_RAM::NR21) >> 6 & 3;
        let pattern = WAVE_PATTERN_DUTY[duty_select as usize];
        let bit = (pattern >> self.channel2.duty_position) & 1;
        // 0 to 15
        let digital =
            if self.channel2.enabled && bit == 1 { self.channel2.envelope.volume } else { 0 };

        let dac_enabled = (self.read_register(APU_RAM::NR22) & 0b11111000) != 0;
        let analog = if dac_enabled { (digital as f32 / 7.5) - 1.0 } else { 0.0 };
        return analog;
    }

    pub fn output_channel3(&self) -> f32 {
        let index = self.channel3.wave_position / 2;
        let nibble = if self.channel3.wave_position % 2 == 0 {
            // Take upper nibble
            (self.wave_ram[index as usize] & 0b11110000) >> 4
        } else {
            self.wave_ram[index as usize] & 0b1111
        };

        let output_level = (self.read_register(APU_RAM::NR32) & 0b1100000) >> 5;
        let nibble = match output_level {
            0 => 0,
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => panic!("Error: unrecognized output level"),
        };

        let dac_enabled = (self.read_register(APU_RAM::NR30) & 0b10000000) != 0;
        let digital = if self.channel3.enabled && dac_enabled { nibble } else { 0 };
        let analog = (digital as f32 / 7.5) - 1.0; // range: -1 to 1
        return analog;
    }

    pub fn output_channel4(&self) -> f32 {
        let digital = if self.channel4.enabled && (self.channel4.lfsr & 1) == 0 {
            self.channel4.envelope.volume
        } else {
            0
        };
        let dac_enabled = (self.read_register(APU_RAM::NR42) & 0b11111000) != 0;
        let analog = if dac_enabled { (digital as f32 / 7.5) - 1.0 } else { 0.0 };
        return analog;
    }

    pub fn trigger_sweep(&mut self) {
        self.channel1.sweep_frequency = (((self.read_register(APU_RAM::NR14)) as i32) & 7) << 8
            | (self.read_register(APU_RAM::NR13) as i32);

        let sweep_pace = (self.read_register(APU_RAM::NR10) & 0b1110000) >> 4;
        let sweep_direction = (self.read_register(APU_RAM::NR10) & 0b1000) >> 3;
        let sweep_step = self.read_register(APU_RAM::NR10) & 0b111;
        self.channel1.sweep_timer = if sweep_pace == 0 { 8 } else { sweep_pace };
        self.channel1.sweep_enabled = sweep_pace != 0 || sweep_step != 0;

        if sweep_step != 0 {
            let new_frequency: i32 = if sweep_direction == 0 {
                self.channel1.sweep_frequency + (self.channel1.sweep_frequency / (1 << sweep_step))
            } else {
                self.channel1.sweep_frequency - (self.channel1.sweep_frequency / (1 << sweep_step))
            };

            if new_frequency > 2047 {
                self.channel1.enabled = false;
            }
        }
    }

    pub fn clock_length_timers(&mut self) {
        if self.channel1.length.clock(self.read_register(APU_RAM::NR14)) {
            self.channel1.enabled = false;
        }
        if self.channel2.length.clock(self.read_register(APU_RAM::NR24)) {
            self.channel2.enabled = false;
        }
        if self.channel3.length.clock(self.read_register(APU_RAM::NR34)) {
            self.channel3.enabled = false;
        }
        if self.channel4.length.clock(self.read_register(APU_RAM::NR44)) {
            self.channel4.enabled = false;
        }
    }

    pub fn clock_sweep(&mut self) {
        self.channel1.sweep_timer -= 1;

        if self.channel1.sweep_timer == 0 {
            let sweep_pace = (self.read_register(APU_RAM::NR10) & 0b1110000) >> 4;
            let sweep_direction = (self.read_register(APU_RAM::NR10) & 0b1000) >> 3;
            let sweep_step = self.read_register(APU_RAM::NR10) & 0b111;
            self.channel1.sweep_timer = if sweep_pace == 0 { 8 } else { sweep_pace };

            if self.channel1.sweep_enabled && sweep_pace != 0 {
                let new_frequency: i32 = if sweep_direction == 0 {
                    self.channel1.sweep_frequency
                        + (self.channel1.sweep_frequency / (1 << sweep_step))
                } else {
                    self.channel1.sweep_frequency
                        - (self.channel1.sweep_frequency / (1 << sweep_step))
                };

                if new_frequency > 2047 {
                    self.channel1.enabled = false;
                } else if sweep_step != 0 {
                    self.channel1.sweep_frequency = new_frequency;
                    self.write_register(APU_RAM::NR13, new_frequency as u8 & 0b11111111);
                    self.write_register(
                        APU_RAM::NR14,
                        ((new_frequency & 0b11100000000) >> 8) as u8,
                    );
                }
            }
        }
    }

    pub fn clock_envelope(&mut self) {
        self.channel1.envelope.clock(self.read_register(APU_RAM::NR12));
        self.channel2.envelope.clock(self.read_register(APU_RAM::NR22));
        self.channel4.envelope.clock(self.read_register(APU_RAM::NR42));
    }
}
