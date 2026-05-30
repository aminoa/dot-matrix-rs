use crate::cart::Cart;
use crate::consts::{CB_OPCODES, CYCLES_PER_FRAME, OPCODES};
use crate::cpu::CPU;
use crate::joypad::Joypad;
use crate::mmu::MMU;
use crate::ppu::PPU;
use std::cell::RefCell;
use std::fs;

pub struct GB {
    pub cpu: CPU,
    pub mmu: MMU,
    pub ppu: PPU,
    pub cart: Cart,
    pub joypad: Joypad,
}

impl GB {
    pub fn new(rom_path: String) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        return GB {
            cpu: CPU::new(),
            mmu: MMU::new(),
            ppu: PPU::new(),
            cart: Cart::from_rom(rom),
            joypad: Joypad::new(),
        };
    }
}
