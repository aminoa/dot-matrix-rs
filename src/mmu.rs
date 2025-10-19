use crate::cart::Cart;
use crate::joypad::Joypad;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MMU {
    pub ram: Vec<u8>,
    pub cart: Cart,
    pub joypad: Rc<RefCell<Joypad>>,
}

impl MMU {
    pub fn new(cart: Cart, joypad: Rc<RefCell<Joypad>>) -> MMU {
        let mut ram = vec![0; 0x10000];
        ram[0xFF00] = 0xCF; // Initialize joypad register with default value (all buttons released)

        return MMU {
            ram: ram,
            cart: cart,
            joypad: joypad,
        };
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => self.joypad.borrow().read(),
            0xFF01 => 0xFF, // Dummy value for serial data register
            0x0..=0x7FFF => self.cart.rom[addr as usize],
            _ => self.ram[addr as usize],
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF00 => self.joypad.borrow_mut().write(value),
            0xFF46 => self.oam_dma_transfer(value),
            0x0..=0x7FFF => (), // Ignore writes to ROM
            _ => self.ram[addr as usize] = value,
        }
    }

    pub fn read_short(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    pub fn write_short(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    // copy 160 bytes to OAM (0xFE00)
    pub fn oam_dma_transfer(&mut self, source_high: u8) {
        // convert XX to XX00
        let source = (source_high as u16) << 8;
        for i in 0x0 as u16..0xA0 as u16 {
            let val = self.read_byte(source + i);
            let dest = 0xFE00 as u16 + i;
            self.ram[dest as usize] = val;
        }
    }
}
