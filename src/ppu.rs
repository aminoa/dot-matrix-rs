use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::cpu::{InterruptBit, CPU};
use crate::mmu::MMU;
use std::cell::RefCell;
use std::rc::Rc;

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

pub enum LCDCBits {
    BgDisplay = 0,
    ObjDisplayEnable = 1,
    ObjDisplayMode = 2,
    BgTileMapDisplaySelect = 3,
    BgAndWindowTileDataSelect = 4,
    WindowDisplayEnable = 5,
    WindowTileMapDisplaySelect = 6,
    LCDDisplayEnable = 7,
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
            current_cycles: 0,
        }
    }

    pub fn update(&mut self, cycles: u32) {
        let scanline = self.mmu.borrow().read_byte(PPUMemory::LY as u16);
        self.current_cycles += cycles;

        match self.current_mode {
            // Mode 2
            PPUMode::OAM => {
                if self.current_cycles > 80 {
                    self.current_cycles -= 80;
                    self.current_mode = PPUMode::VRAM;
                }
            }
            // Mode 3
            PPUMode::VRAM => {
                if self.current_cycles > 172 {
                    self.current_cycles -= 172;
                    self.current_mode = PPUMode::HBlank;
                    self.render_tile_data();
                }
            }
            // Mode 0
            PPUMode::HBlank => {
                if self.current_cycles > 204 {
                    self.current_cycles -= 204;
                    if scanline == SCREEN_HEIGHT as u8 {
                        // TODO: more interrupt sources
                        self.cpu
                            .borrow_mut()
                            .request_interrupt(InterruptBit::VBlank);
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, 0);
                        self.current_mode = PPUMode::VBlank;
                    } else {
                        self.mmu
                            .borrow_mut()
                            .write_byte(PPUMemory::LY as u16, scanline + 1);
                        self.current_mode = PPUMode::OAM;
                    }
                }
            }
            // Mode 1
            PPUMode::VBlank => {
                if self.current_cycles > 456 {
                    if scanline == SCREEN_HEIGHT as u8 + 10 {
                        self.mmu.borrow_mut().write_byte(PPUMemory::LY as u16, 0);
                        self.current_mode = PPUMode::OAM;
                    } else {
                        self.mmu
                            .borrow_mut()
                            .write_byte(PPUMemory::LY as u16, scanline + 1);
                    }

                    self.current_cycles -= 456;
                }
            }
        }
    }

    pub fn draw_scanline(&mut self, scanline: u8) {
        // draw background scanline
        // let lcdc = self.mmu.borrow().read_byte(PPUMemory::LCDC as u16);
        // if (lcdc & (1 << LCDCBits::BgDisplay as u8)) != 0 {
        //     self.draw_background_scanline(scanline);
        // }
    }

    pub fn render_tile_data(&mut self) {
        self.framebuffer.fill(0xFF);

        // test pattern

        // for y in 0..144 {
        //     for x in 0..160 {
        //         if (x / 8 + y / 8) % 2 == 0 {
        //             self.framebuffer[y * 160 + x] = 0xFF; // White
        //         } else {
        //             self.framebuffer[y * 160 + x] = 0x00; // Black
        //         }
        //     }
        // }

        // VRAM tile data is stored at 0x8000-0x97FF
        // Each tile is 8x8 pixels, with 2 bits per pixel (16 bytes per tile)
        // We can display up to 384 tiles (24 rows of 16 tiles)

        let tiles_per_row = 16; // 16 tiles per row
        let tile_size = 8; // 8x8 pixels

        for tile_index in 0..384 {
            let tile_x = (tile_index % tiles_per_row) * tile_size;
            let tile_y = (tile_index / tiles_per_row) * tile_size;

            // Skip tiles that would be rendered outside the framebuffer
            if tile_y >= 144 || tile_x >= 160 {
                continue;
            }

            // Tile data starts at 0x8000
            let tile_addr = 0x8000 + (tile_index * 16);

            // Each tile is 8 rows of 8 pixels
            for row in 0..8 {
                let addr = tile_addr + (row * 2);
                let low_byte = self.mmu.borrow().read_byte(addr as u16);
                let high_byte = self.mmu.borrow().read_byte((addr + 1) as u16);

                for col in 0..8 {
                    // Each pixel uses 2 bits (one from each byte)
                    // Bit 7 is leftmost pixel, bit 0 is rightmost
                    let bit_position = 7 - col;
                    let low_bit = (low_byte >> bit_position) & 0x01;
                    let high_bit = (high_byte >> bit_position) & 0x01;
                    let color_id = (high_bit << 1) | low_bit;

                    // Read the BGP (Background Palette) register
                    let bgp = self.mmu.borrow().read_byte(PPUMemory::BGP as u16);

                    // Extract the actual color from the palette
                    // Each 2 bits in the palette represent a color
                    let palette_color = (bgp >> (color_id * 2)) & 0x03;

                    // Convert the palette color to grayscale
                    let color = match palette_color {
                        0 => 0xFF, // White
                        1 => 0xAA, // Light gray
                        2 => 0x55, // Dark gray
                        3 => 0x00, // Black
                        _ => 0xFF, // Should never happen
                    };

                    // Calculate position in framebuffer
                    let x = tile_x + col;
                    let y = tile_y + row;

                    // Make sure we're within bounds
                    if x < 160 && y < 144 {
                        self.framebuffer[y * 160 + x] = color;
                    }
                }
            }
        }
    }

    // pub fn draw_background_scanline(&mut self, scanline: u8) {}
}
