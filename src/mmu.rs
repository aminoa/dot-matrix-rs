use crate::cart::Cart;
use crate::joypad::Joypad;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct MMU {
    pub ram: Vec<u8>,
}

impl MMU {
    pub fn new() -> MMU {
        let mut ram = vec![0; 0x10000];
        ram[0xFF00] = 0xCF; // Initialize joypad register with default value (all buttons released)

        return MMU { ram };
    }

    pub fn read_byte(&self, addr: u16, cart: &Cart, joypad: &Joypad) -> u8 {
        match addr {
            0x0..=0x7FFF => cart.read_rom(addr),
            0xA000..0xBFFF => cart.read_ram(addr), // if this exists
            0xFF00 => joypad.read(),
            0xFF01 => 0xFF, // Dummy value for serial data register
            // 0xFF10..0xFF26 => apu.read_register(addr),
            _ => self.ram[addr as usize],
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8, cart: &mut Cart, joypad: &mut Joypad) {
        match addr {
            0x0000..0x7FFF => cart.write_rom(addr, val),
            0xA000..0xBFFF => cart.write_ram(addr, val),
            0xFF00 => joypad.write(val),
            0xFF46 => self.oam_dma_transfer(val, cart, joypad),
            _ => self.ram[addr as usize] = val,
        }
    }

    pub fn read_short(&self, addr: u16, cart: &Cart, joypad: &Joypad) -> u16 {
        (self.read_byte(addr, cart, joypad) as u16)
            | ((self.read_byte(addr + 1, cart, joypad) as u16) << 8)
    }

    pub fn write_short(&mut self, addr: u16, val: u16, cart: &mut Cart, joypad: &mut Joypad) {
        self.write_byte(addr, (val & 0xFF) as u8, cart, joypad);
        self.write_byte(addr + 1, (val >> 8) as u8, cart, joypad);
    }

    // copy 160 bytes to OAM (0xFE00)
    pub fn oam_dma_transfer(&mut self, source_high: u8, cart: &Cart, joypad: &Joypad) {
        // convert XX to XX00
        let source = (source_high as u16) << 8;
        for i in 0x0 as u16..0xA0 as u16 {
            let val = self.read_byte(source + i, cart, joypad);
            let dest = 0xFE00 as u16 + i;
            self.ram[dest as usize] = val;
        }
    }

    pub fn saveram(&mut self, rom_path: &String, cart: &Cart) {
        let rom_path = Path::new(rom_path);
        let mut save_path = PathBuf::from(rom_path);
        save_path.set_extension("sav");
        fs::write(&save_path, &cart.ram).expect("Error: unable to write RAM contents")
    }
}
