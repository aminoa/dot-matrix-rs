use crate::cart::Cart;
use crate::joypad::Joypad;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;

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
            _ => self.ram[addr as usize],
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8, cart: &mut Cart, joypad: &mut Joypad) {
        match addr {
            0x0..0x1FFF => cart.enable_ram(value),
            0x2000..0x3FFF => cart.select_rom_bank(value),

            0xFF00 => joypad.write(value),
            0xFF46 => self.oam_dma_transfer(value, cart, joypad),
            0x0..=0x7FFF => (), // Ignore writes to ROM
            _ => self.ram[addr as usize] = value,
        }
    }

    pub fn read_short(&self, addr: u16, cart: &Cart, joypad: &Joypad) -> u16 {
        (self.read_byte(addr, cart, joypad) as u16)
            | ((self.read_byte(addr + 1, cart, joypad) as u16) << 8)
    }

    pub fn write_short(&mut self, addr: u16, value: u16, cart: &mut Cart, joypad: &mut Joypad) {
        self.write_byte(addr, (value & 0xFF) as u8, cart, joypad);
        self.write_byte(addr + 1, (value >> 8) as u8, cart, joypad);
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

    pub fn savestate(&self) {
        // dump the MMU to a file
        // MMU: dump everything from 0x8000 to 0xFFFF
        let mut file = File::create("savestate.mmu").unwrap();
        file.write_all(&self.ram[0x8000..0x10000]).unwrap();
        println!("Savestate written to savestate.mmu");
    }

    pub fn loadstate(&mut self) {
        let mut file = File::open("savestate.mmu").unwrap();
        let mut buffer = vec![0; 0x10000 - 0x8000];
        file.read_exact(&mut buffer).unwrap();

        for (i, &byte) in buffer.iter().enumerate() {
            self.ram[0x8000 + i] = byte;
        }
    }
}
