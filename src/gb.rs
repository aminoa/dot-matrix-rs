use crate::apu::APU;
use crate::cart::Cart;
use crate::consts::{CB_OPCODES, CYCLES_PER_FRAME, OPCODES};
use crate::cpu::CPU;
use crate::joypad::Joypad;
use crate::mmu::MMU;
use crate::ppu::PPU;
use std::cell::RefCell;
use std::fs;

pub struct GB {
    pub apu: APU,
    pub cpu: CPU,
    pub mmu: MMU,
    pub ppu: PPU,
    pub cart: Cart,
    pub joypad: Joypad,
    pub current_cycles: u32,
}

impl GB {
    pub fn new(rom_path: String) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        return GB {
            apu: APU::new(),
            cpu: CPU::new(),
            mmu: MMU::new(),
            ppu: PPU::new(),
            cart: Cart::from_rom(rom),
            joypad: Joypad::new(),
            current_cycles: 0,
        };
    }

    pub fn step(&mut self) {
        let instruction = self.mmu.read_byte(self.cpu.pc, &self.cart, &self.joypad);

        let instruction_cycles =
            self.cpu.execute(instruction, &mut self.mmu, &mut self.cart, &mut self.joypad);
        self.cpu.check_interrupts(&mut self.mmu, &mut self.cart, &mut self.joypad);
        self.cpu.update_timers(
            instruction_cycles as u32,
            &mut self.mmu,
            &mut self.cart,
            &mut self.joypad,
        );
        self.ppu.update(
            instruction_cycles as u32,
            &mut self.mmu,
            &mut self.cpu,
            &mut self.cart,
            &mut self.joypad,
        );
        self.apu.update();

        self.current_cycles += instruction_cycles as u32;

        if self.mmu.read_byte(0xFF02, &self.cart, &self.joypad) == 0x81 {
            print!("{}", self.mmu.read_byte(0xFF01, &self.cart, &self.joypad) as char);
            self.mmu.write_byte(0xFF02, 0, &mut self.cart, &mut self.joypad);
        }
    }
}
