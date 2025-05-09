use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::mmu::MMU;
use crate::cpu::{CPU, InterruptBit};
use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
    pub cpu: Rc<RefCell<CPU>>,
    pub mmu: Rc<RefCell<MMU>>,
    pub framebuffer: [[u8; 144]; 160],
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

pub enum LCDCBit {
    LCDEnabled = 7,
    WindowTileMap = 6,
    WindowEnabled = 5,
    BGTileMap = 4,
    BGTileData = 3,
    OBJSize = 2,
    OBJEnabled = 1,
    BGEnabled = 0,
}

pub enum STATBit {
    LYC = 6,
    Mode2OAM = 5,
    Mode1VBlank = 4,
    Mode0HBlank = 3,
    CoincidenceInterrupt = 2,
    PPUModeBit1 = 1,
    PPUModeBit0 = 0,
}

// pub enum STATBit {
//     Mode = 3,
//     HBlank = 0,
//     VBlank = 1,
//     OAM = 2,
// }

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
        let framebuffer = [[0u8; 144]; 160];

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
                // println!("OAM");
                if self.current_cycles > 80 {
                    self.current_cycles -= 80;

                    self.current_mode = PPUMode::VRAM;
                }
            },
            PPUMode::VRAM => {
                // println!("VRAM");
                if self.current_cycles > 172 {
                    self.current_cycles -= 172;

                    self.current_mode = PPUMode::HBlank;
                    // self.draw_scanline();
                    // update background layer
                    // update window layer
                    // update sprite layer
                }

            },
            PPUMode::HBlank => {
                // println!("HBlank");
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
                // println!("VBlank");                 
                if self.current_cycles > 456 {
                    self.current_cycles -= 456;

                    if scanline == SCREEN_HEIGHT as u8 + 10 {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, 0);
                        self.current_mode = PPUMode::OAM;
                    } else {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, scanline + 1);
                    }
                }
            },
        }
    }

    // pub fn draw_scanline(&mut self) {
        // draw_background();
        // draw_window();
    // }

    #[cfg(debug_assertions)]
    pub fn draw_tile(&mut self) { 
        // Render tiles using 8000 as base pointer to the framebuffer
        // 8x8 tiles make up the screen, 8x8 pixels make up a tile
        let base_pointer = 0x8000;

        for i in 0..8 {
            let pixel_part_one = self.mmu.borrow().read_byte(base_pointer + i * 2);
            let pixel_part_two = self.mmu.borrow().read_byte(base_pointer + i * 2 + 1);
            for j in 0..8 {
                let pixel = ((pixel_part_one >> (7 - j)) & 1) | (((pixel_part_two >> (7 - j)) & 1) << 1);
                let color = match pixel {
                    0 => GameBoyColor::White,
                    1 => GameBoyColor::LightGray,
                    2 => GameBoyColor::DarkGray,
                    3 => GameBoyColor::Black,
                    _ => unreachable!(),
                };
                self.framebuffer[i as usize][j as usize] = color.to_u8();
            }
        }

    }
}