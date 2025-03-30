use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::mmu::MMU;
use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
    pub mmu: Rc<RefCell<MMU>>,
    pub framebuffer: [[u8; 144]; 160]

}

pub enum GameBoyColor {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl GameBoyColor {
    pub fn to_u8(&self) -> u8 {
        match self {
            GameBoyColor::White => 0xFF,
            GameBoyColor::LightGray => 0xAA,
            GameBoyColor::DarkGray => 0xFF,
            GameBoyColor::Black => 0x00,
        }
    }
}

impl PPU {
    pub fn new(mmu: Rc<RefCell<MMU>>) -> PPU {
        let framebuffer = [[0u8; 144]; 160];

        PPU {
            mmu: mmu,
            framebuffer: framebuffer
        }
    }

    pub fn update(&mut self, cycles: u32) {

        self.draw_scanline(cycles);

        // Each scanline goes through 3 modes
        // The last mode is Vblank

        // Mode 2 - OAM fetch

        // Mode 3
        // update background layer
        // update window layer
        // update sprite layer

        // Mode 0 - HBlank
        // Mode 1 - VBlank
    }

    pub fn draw_scanline(&mut self, cycles: u32) {
        for y in 0..SCREEN_HEIGHT as usize {
            self.framebuffer[10][y] = GameBoyColor::White.to_u8();
        }

        for x in 0..SCREEN_WIDTH as usize {
            self.framebuffer[x][10] = GameBoyColor::White.to_u8();
        }
    }
}