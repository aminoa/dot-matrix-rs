use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::cpu::{InterruptBit, CPU};
use crate::mmu::MMU;
use std::cell::RefCell;
use std::marker;
use std::process::exit;
use std::rc::Rc;
use std::thread::JoinHandle;

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
    SCY = 0xFF42, //bg
    SCX = 0xFF43, //bg
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A, //window
    WX = 0xFF4B, //window
}
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
}

pub enum LCDCBits {
    BackgroundWindowEnable = 0,
    ObjectDisplayEnable = 1,
    ObjectDisplaySize = 2,
    BackgroundTileMapArea = 3,
    BackgroundAndWindowTileDataSelect = 4,
    WindowDisplayEnable = 5,
    WindowTileMapDisplaySelect = 6,
    LCDDisplayEnable = 7,
}

// Color constants for better readability
pub const COLOR_WHITE: u8 = 0xFF;
pub const COLOR_LIGHT_GRAY: u8 = 0xAA;
pub const COLOR_DARK_GRAY: u8 = 0x55;
pub const COLOR_BLACK: u8 = 0x00;

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
                    // Render the current scanline
                    if (self.mmu.borrow().read_byte(PPUMemory::LCDC as u16)
                        & (1 << LCDCBits::LCDDisplayEnable as u8))
                        != 0
                    {
                        self.draw_scanline(scanline);
                    }
                }
            }
            // Mode 0
            PPUMode::HBlank => {
                if self.current_cycles > 204 {
                    self.current_cycles -= 204;
                    if scanline == SCREEN_HEIGHT as u8 - 1 {
                        // TODO: more interrupt sources
                        self.cpu
                            .borrow_mut()
                            .request_interrupt(InterruptBit::VBlank);
                        self.mmu
                            .borrow_mut()
                            .write_byte(PPUMemory::LY as u16, scanline + 1);
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
                    if scanline == SCREEN_HEIGHT as u8 + 9 {
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
        let lcdc = self.mmu.borrow().read_byte(PPUMemory::LCDC as u16);

        if (lcdc & (1 << LCDCBits::LCDDisplayEnable as u8)) == 0 {
            return;
        }

        if (lcdc & (1 << LCDCBits::BackgroundWindowEnable as u8)) != 0 {
            self.draw_background_scanline(scanline);
        }

        if (lcdc & (1 << LCDCBits::WindowDisplayEnable as u8)) != 0 {
            self.draw_window_scanline(scanline);
        }

        if (lcdc & (1 << LCDCBits::ObjectDisplayEnable as u8)) != 0 {
            self.draw_sprites_scanline(scanline);
        }
    }

    pub fn draw_background_scanline(&mut self, scanline: u8) {
        for x in 0..SCREEN_WIDTH as u16 {
            // getting tile map and data base
            let lcdc = self.mmu.borrow().read_byte(PPUMemory::LCDC as u16);
            let tile_map_base_bit = (lcdc >> LCDCBits::BackgroundTileMapArea as u8) & 1;

            let tile_map_base: u16 = if tile_map_base_bit == 0 {
                0x9800
            } else {
                0x9C00
            };

            let tile_data_base_bit =
                (lcdc >> LCDCBits::BackgroundAndWindowTileDataSelect as u8) & 1;

            let tile_data_base: u16 = if tile_data_base_bit == 0 {
                0x8800
            } else {
                0x8000
            };

            // 32 tiles per row so going down one row requires * 32, / 8 because each tile is 8 * 8 px
            let scx = self.mmu.borrow().read_byte(PPUMemory::SCX as u16);
            let scy = self.mmu.borrow().read_byte(PPUMemory::SCY as u16);
            let background_x = x.wrapping_add(scx as u16) % 256;
            let background_y = scanline.wrapping_add(scy) as u16 % 256;

            let tile_map_row_offset = ((background_y / 8) * 32) as u16;
            let tile_map_col_offset = (background_x / 8) as u16;

            let tile_map_offset: u16 = tile_map_row_offset + tile_map_col_offset;
            let tile_index = self.mmu.borrow().read_byte(tile_map_base + tile_map_offset);

            // 8800 + (127 + 128) * 16 = 97F0 (can grab the last 2 bytes of memory for tile data)
            // 8800 + (-128 + 128) * 16 = 8800
            let tile_data_address: u16 = if tile_data_base == 0x8000 {
                tile_data_base + (tile_index as u16 * 16)
            } else {
                let signed_index = tile_index as i8;
                tile_data_base + ((signed_index as i16 + 128) * 16) as u16
            };

            let tile_data_line = (background_y % 8) as u16; //within the tile, the line looked at

            // 2BPP calculations below to get a pixel
            // Ex. 8000 + (2F * 0x10) = 82F0
            // Get the two bytes for the line (there are 16 bytes per tile, 2 bytes per line)
            let tile_data_byte_1 = self
                .mmu
                .borrow()
                .read_byte(tile_data_address + (tile_data_line * 2));
            let tile_data_byte_2 = self
                .mmu
                .borrow()
                .read_byte(tile_data_address + (tile_data_line * 2 + 1));

            // Get the two bits for the pixel within the line (that's why x is used)
            let tile_data_byte_index = 7 - (background_x % 8);
            let tile_data_bit_1 = (tile_data_byte_1 >> tile_data_byte_index) & 1;
            let tile_data_bit_2 = (tile_data_byte_2 >> tile_data_byte_index) & 1;

            // originally called tile_data_bit_color, values from 0 - 3
            let color_index = (tile_data_bit_2 << 1) | tile_data_bit_1;
            let palette = self.mmu.borrow().read_byte(PPUMemory::BGP as u16);

            let color = match (palette >> (color_index * 2)) & 0b11 {
                0 => COLOR_WHITE,
                1 => COLOR_LIGHT_GRAY,
                2 => COLOR_DARK_GRAY,
                3 => COLOR_BLACK,
                _ => COLOR_WHITE,
            };

            self.framebuffer[((scanline as u32 * SCREEN_WIDTH) + x as u32) as usize] = color;
        }
    }

    pub fn draw_window_scanline(&mut self, scanline: u8) {
        for x in 0..SCREEN_WIDTH as u16 {
            let wx = self.mmu.borrow().read_byte(PPUMemory::WX as u16);
            let wy = self.mmu.borrow().read_byte(PPUMemory::WY as u16);

            if scanline < wy || x + 7 < wx as u16 {
                continue;
            }

            let lcdc = self.mmu.borrow().read_byte(PPUMemory::LCDC as u16);
            let tile_map_base_bit = (lcdc >> LCDCBits::WindowTileMapDisplaySelect as u8) & 1;

            let tile_map_base: u16 = if tile_map_base_bit == 0 {
                0x9800
            } else {
                0x9C00
            };

            let tile_data_base_bit =
                (lcdc >> LCDCBits::BackgroundAndWindowTileDataSelect as u8) & 1;

            let tile_data_base: u16 = if tile_data_base_bit == 0 {
                0x8800
            } else {
                0x8000
            };

            // let background_x = x.wrapping_add(scx as u16) % 256;
            // let background_y = scanline.wrapping_add(scy) as u16 % 256;
            let window_y = scanline - wy;
            let window_x = x - wx as u16;

            let tile_map_row_offset = ((window_x / 8) * 32) as u16;
            let tile_map_col_offset = (window_y / 8) as u16;

            let tile_map_offset: u16 = tile_map_row_offset + tile_map_col_offset;
            let tile_index = self.mmu.borrow().read_byte(tile_map_base + tile_map_offset);

            // 8800 + (127 + 128) * 16 = 97F0 (can grab the last 2 bytes of memory for tile data)
            // 8800 + (-128 + 128) * 16 = 8800
            let tile_data_address: u16 = if tile_data_base == 0x8000 {
                tile_data_base + (tile_index as u16 * 16)
            } else {
                let signed_index = tile_index as i8;
                tile_data_base + ((signed_index as i16 + 128) * 16) as u16
            };

            let tile_data_line = (window_y % 8) as u16; //within the tile, the line looked at

            // 2BPP calculations below to get a pixel
            // Ex. 8000 + (2F * 0x10) = 82F0
            // Get the two bytes for the line (there are 16 bytes per tile, 2 bytes per line)
            let tile_data_byte_1 = self
                .mmu
                .borrow()
                .read_byte(tile_data_address + (tile_data_line * 2));
            let tile_data_byte_2 = self
                .mmu
                .borrow()
                .read_byte(tile_data_address + (tile_data_line * 2 + 1));

            // Get the two bits for the pixel within the line (that's why x is used)
            let tile_data_byte_index = 7 - (window_x % 8);
            let tile_data_bit_1 = (tile_data_byte_1 >> tile_data_byte_index) & 1;
            let tile_data_bit_2 = (tile_data_byte_2 >> tile_data_byte_index) & 1;

            // originally called tile_data_bit_color, values from 0 - 3
            let color_index = (tile_data_bit_2 << 1) | tile_data_bit_1;
            let palette = self.mmu.borrow().read_byte(PPUMemory::BGP as u16);

            let color = match (palette >> (color_index * 2)) & 0b11 {
                0 => COLOR_WHITE,
                1 => COLOR_LIGHT_GRAY,
                2 => COLOR_DARK_GRAY,
                3 => COLOR_BLACK,
                _ => COLOR_WHITE,
            };

            self.framebuffer[((scanline as u32 * SCREEN_WIDTH) + x as u32) as usize] = color;
        }
    }

    pub fn draw_sprites_scanline(&mut self, scanline: u8) {}
}
