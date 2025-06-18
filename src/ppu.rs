use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::mmu::MMU;
use crate::cpu::{CPU, InterruptBit};
use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
    pub cpu: Rc<RefCell<CPU>>,
    pub mmu: Rc<RefCell<MMU>>,
    pub framebuffer: [u8; 144 * 160],
    pub current_mode: PPUMode,
    pub current_cycles: u32,
}

pub enum PPUMemory {
    LCDC = 0xFF40,
    STAT = 0xFF41,
    SCY = 0xFF42,
    SCX = 0xFF43,
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A,
    WX = 0xFF4B,
}

pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
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
    pub fn new(mmu: Rc<RefCell<MMU>>, cpu: Rc<RefCell<CPU>>) -> PPU {
        let framebuffer = [0xFFu8; 144 * 160];

        PPU {
            cpu: cpu,
            mmu: mmu,
            framebuffer: framebuffer,
            current_mode: PPUMode::OAM,
            current_cycles: 0
        }
    }

    pub fn update(&mut self, cycles: u32) {
        let scanline = self.mmu.borrow().read_byte(PPUMemory::LY as u16);
        self.current_cycles += cycles;

        match self.current_mode {
            PPUMode::OAM => {
                if self.current_cycles > 80 {
                    self.current_cycles -= 80;
                    self.current_mode = PPUMode::VRAM;
                }
            },
            PPUMode::VRAM => {
                if self.current_cycles > 172 {
                    self.current_cycles -= 172;
                    self.current_mode = PPUMode::HBlank;
                    // self.draw_background_scanline(scanline);

                    // update background layer
                    // update window layer
                    // update sprite layer
                }

            },
            PPUMode::HBlank => {
                if self.current_cycles > 204 {
                    self.current_cycles -= 204;
                    if scanline == SCREEN_HEIGHT as u8 {
                        // TODO: more interrupt sources
                        self.cpu.borrow_mut().request_interrupt(InterruptBit::VBlank);
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, 0);
                        self.current_mode = PPUMode::VBlank;

                    } else {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, scanline + 1);
                        self.current_mode = PPUMode::OAM;
                    }
                }
            },
            PPUMode::VBlank => {
                if self.current_cycles > 456 {
                    if scanline == SCREEN_HEIGHT as u8 + 10 {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, 0);
                        self.current_mode = PPUMode::OAM;
                    } else {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, scanline + 1);
                    }

                    self.current_cycles -= 456;
                }
            },
        }
    }

    // pub fn draw_scanline(&mut self) {
    //     let scanline = self.mmu.borrow().read_byte(PPUMemory::LY as u16);
    //     if scanline < SCREEN_HEIGHT as u8 {
    //         // Draw the current scanline
    //         self.draw_background_scanline();
    //     }
    //     // self.draw_window_scanline();
    //     // self.draw_sprite_scanline();
    // }

    // pub fn draw_background_scanline(&mut self) {
    //     let lcdc = self.mmu.borrow().read_byte(PPUMemory::LCDC as u16);
        
    // }

    #[cfg(debug_assertions)]
    pub fn draw_tileset(&mut self) {
        
    }
}