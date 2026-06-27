use crate::cart::Cart;
use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::cpu::{InterruptBit, CPU};
use crate::joypad::Joypad;
use crate::mmu::MMU;

pub struct PPU {
    pub framebuffer: [u8; 144 * 160],
    pub current_mode: PPUMode,
    pub current_cycles: u32,
    pub stat_line: bool,
    pub window_line_counter: u8,
}

pub enum PPUMemory {
    LCDC = 0xFF40,
    STAT = 0xFF41,
    SCY = 0xFF42, //background
    SCX = 0xFF43, //background
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A, //window
    WX = 0xFF4B, //window
}

#[derive(Clone)]
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
}

pub enum OAMAttributesBits {
    Priority = 7,
    YFlip = 6,
    XFlip = 5,
    PaletteNumber = 4,
}

pub enum LCDCBits {
    BackgroundWindowEnable = 0,
    ObjectDisplayEnable = 1,
    ObjectSize = 2,
    BackgroundTileMapArea = 3,
    BackgroundAndWindowTileDataSelect = 4,
    WindowDisplayEnable = 5,
    WindowTileMapDisplaySelect = 6,
    LCDDisplayEnable = 7,
}

pub enum LCDStatBits {
    PPUModeFlag0 = 0,
    PPUModeFlag1 = 1,
    LYCEqualsLY = 2,
    Mode0IntSelect = 3, // H-Blank
    Mode1IntSelect = 4, // V-Blank
    Mode2IntSelect = 5, // OAM Scan
    LCDIntSelect = 6,
}

// Color constants for better readability
pub const COLOR_WHITE: u8 = 0xFF;
pub const COLOR_LIGHT_GRAY: u8 = 0xAA;
pub const COLOR_DARK_GRAY: u8 = 0x55;
pub const COLOR_BLACK: u8 = 0x00;

impl PPU {
    pub fn new() -> PPU {
        let framebuffer = [0xFFu8; 144 * 160];

        PPU {
            framebuffer: framebuffer,
            current_mode: PPUMode::OAM,
            current_cycles: 0,
            stat_line: false,
            window_line_counter: 0,
        }
    }

    pub fn update(
        &mut self,
        cycles: u32,
        mmu: &mut MMU,
        cpu: &mut CPU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        let scanline = mmu.read_byte(PPUMemory::LY as u16, cart, joypad);
        let stat = mmu.read_byte(PPUMemory::STAT as u16, cart, joypad);

        self.update_stat(scanline, mmu, cpu, cart, joypad);

        self.current_cycles += cycles;

        match self.current_mode {
            // Mode 2
            PPUMode::OAM => {
                if self.current_cycles > 80 {
                    self.current_cycles -= 80;
                    self.current_mode = PPUMode::VRAM;
                    self.update_stat(scanline, mmu, cpu, cart, joypad);
                }
            }
            // Mode 3
            PPUMode::VRAM => {
                if self.current_cycles > 172 {
                    self.current_cycles -= 172;
                    self.current_mode = PPUMode::HBlank;
                    self.draw_scanline(scanline, mmu, cart, joypad);
                }
            }
            // Mode 0
            PPUMode::HBlank => {
                if self.current_cycles > 204 {
                    self.update_stat(scanline, mmu, cpu, cart, joypad);
                    self.current_cycles -= 204;
                    if scanline == SCREEN_HEIGHT as u8 - 1 {
                        cpu.request_interrupt(InterruptBit::VBlank, mmu, cart, joypad);
                        mmu.write_byte(PPUMemory::LY as u16, scanline + 1, cart, joypad);
                        self.current_mode = PPUMode::VBlank;
                        self.window_line_counter = 0;
                    } else {
                        mmu.write_byte(PPUMemory::LY as u16, scanline + 1, cart, joypad);
                        self.current_mode = PPUMode::OAM;
                    }
                }
            }
            // Mode 1
            PPUMode::VBlank => {
                if self.current_cycles > 456 {
                    if scanline == SCREEN_HEIGHT as u8 + 9 {
                        mmu.write_byte(PPUMemory::LY as u16, 0, cart, joypad);
                        self.current_mode = PPUMode::OAM;
                    } else {
                        mmu.write_byte(PPUMemory::LY as u16, scanline + 1, cart, joypad);
                    }

                    self.update_stat(scanline, mmu, cpu, cart, joypad);
                    self.current_cycles -= 456;
                }
            }
        }
    }

    pub fn update_stat(
        &mut self,
        scanline: u8,
        mmu: &mut MMU,
        cpu: &mut CPU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        let mut stat = mmu.read_byte(PPUMemory::STAT as u16, cart, joypad);

        // Update read-only stat information
        // LYC = LY, bit 2 check
        let lyc = mmu.read_byte(PPUMemory::LYC as u16, cart, joypad);
        if scanline == lyc {
            stat |= 1 << LCDStatBits::LYCEqualsLY as u8;
            // if (stat & (1 << LCDStatBits::LCDIntSelect as u8)) != 0 {
            // }
        } else {
            stat &= !(1 << LCDStatBits::LYCEqualsLY as u8);
        }
        let mode = self.current_mode.clone() as u8;

        // bit 1 set
        stat &= !0b11; // Clear mode bits
        stat |= self.current_mode.clone() as u8;
        mmu.write_byte(PPUMemory::STAT as u16, stat, cart, joypad);
        let current_stat_line = (mode == PPUMode::HBlank as u8
            && (stat & (1 << LCDStatBits::Mode0IntSelect as u8)) != 0)
            || (mode == PPUMode::VBlank as u8
                && (stat & (1 << LCDStatBits::Mode1IntSelect as u8)) != 0)
            || (mode == PPUMode::OAM as u8
                && (stat & (1 << LCDStatBits::Mode2IntSelect as u8)) != 0)
            || (scanline == lyc && (stat & (1 << LCDStatBits::LCDIntSelect as u8)) != 0);

        if !self.stat_line && current_stat_line {
            cpu.request_interrupt(InterruptBit::STAT, mmu, cart, joypad);
        }

        self.stat_line = current_stat_line;
    }

    pub fn draw_scanline(
        &mut self,
        scanline: u8,
        mmu: &mut MMU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        let lcdc = mmu.read_byte(PPUMemory::LCDC as u16, cart, joypad);

        if (lcdc & (1 << LCDCBits::LCDDisplayEnable as u8)) == 0 {
            return;
        }

        if (lcdc & (1 << LCDCBits::BackgroundWindowEnable as u8)) != 0 {
            self.draw_background_scanline(scanline, mmu, cart, joypad);
        }
        if (lcdc & (1 << LCDCBits::WindowDisplayEnable as u8)) != 0 {
            self.draw_window_scanline(scanline, mmu, cart, joypad);
        }

        if (lcdc & (1 << LCDCBits::ObjectDisplayEnable as u8)) != 0 {
            self.draw_sprites_scanline(scanline, mmu, cart, joypad);
        }
    }

    pub fn draw_background_scanline(
        &mut self,
        scanline: u8,
        mmu: &mut MMU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        for x in 0..SCREEN_WIDTH as u16 {
            // getting tile map and data base
            let lcdc = mmu.read_byte(PPUMemory::LCDC as u16, cart, joypad);

            let tile_map_base_bit = (lcdc >> LCDCBits::BackgroundTileMapArea as u8) & 1;

            let tile_map_base: u16 = if tile_map_base_bit == 0 { 0x9800 } else { 0x9C00 };

            let tile_data_base_bit =
                (lcdc >> LCDCBits::BackgroundAndWindowTileDataSelect as u8) & 1;

            let tile_data_base: u16 = if tile_data_base_bit == 0 { 0x8800 } else { 0x8000 };

            // 32 tiles per row so going down one row requires * 32, / 8 because each tile is 8 * 8 px
            let scx = mmu.read_byte(PPUMemory::SCX as u16, cart, joypad);
            let scy = mmu.read_byte(PPUMemory::SCY as u16, cart, joypad);
            let background_x = x.wrapping_add(scx as u16) % 256;
            let background_y = scanline.wrapping_add(scy) as u16 % 256;

            let tile_map_row_offset = ((background_y / 8) * 32) as u16;
            let tile_map_col_offset = (background_x / 8) as u16;

            let tile_map_offset: u16 = tile_map_row_offset + tile_map_col_offset;
            let tile_index = mmu.read_byte(tile_map_base + tile_map_offset, cart, joypad);

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
            let tile_data_byte_1 =
                mmu.read_byte(tile_data_address + (tile_data_line * 2), cart, joypad);
            let tile_data_byte_2 =
                mmu.read_byte(tile_data_address + (tile_data_line * 2 + 1), cart, joypad);

            // Get the two bits for the pixel within the line (that's why x is used), bits go from 7 - 0
            let tile_data_byte_index = 7 - (background_x % 8);
            let tile_data_bit_1 = (tile_data_byte_1 >> tile_data_byte_index) & 1;
            let tile_data_bit_2 = (tile_data_byte_2 >> tile_data_byte_index) & 1;

            // originally called tile_data_bit_color, values from 0 - 3
            let color_index = (tile_data_bit_2 << 1) | tile_data_bit_1;
            let palette = mmu.read_byte(PPUMemory::BGP as u16, cart, joypad);

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

    pub fn draw_window_scanline(
        &mut self,
        scanline: u8,
        mmu: &mut MMU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        let wx = mmu.read_byte(PPUMemory::WX as u16, cart, joypad);
        let wy = mmu.read_byte(PPUMemory::WY as u16, cart, joypad);

        if scanline < wy || wx >= 166 {
            return;
        }

        let window_y = self.window_line_counter;
        self.window_line_counter += 1;

        for x in 0..SCREEN_WIDTH as u16 {
            if x + 7 < wx as u16 {
                continue;
            }

            let lcdc = mmu.read_byte(PPUMemory::LCDC as u16, cart, joypad);
            let tile_map_base_bit = (lcdc >> LCDCBits::WindowTileMapDisplaySelect as u8) & 1;

            let tile_map_base: u16 = if tile_map_base_bit == 0 { 0x9800 } else { 0x9C00 };

            let tile_data_base_bit =
                (lcdc >> LCDCBits::BackgroundAndWindowTileDataSelect as u8) & 1;

            let tile_data_base: u16 = if tile_data_base_bit == 0 { 0x8800 } else { 0x8000 };

            let window_x = x + 7 - wx as u16;
            let tile_map_row_offset = ((window_y as u16 / 8) * 32) as u16;
            let tile_map_col_offset = (window_x as u16 / 8) as u16;

            let tile_map_offset: u16 = tile_map_row_offset + tile_map_col_offset;
            let tile_index = mmu.read_byte(tile_map_base + tile_map_offset, cart, joypad);

            let tile_data_address: u16 = if tile_data_base == 0x8000 {
                tile_data_base + (tile_index as u16 * 16)
            } else {
                let signed_index = tile_index as i8;
                tile_data_base + ((signed_index as i16 + 128) * 16) as u16
            };

            let tile_data_line = (window_y % 8) as u16; //within the tile, the line looked at

            let tile_data_byte_1 =
                mmu.read_byte(tile_data_address + (tile_data_line * 2), cart, joypad);
            let tile_data_byte_2 =
                mmu.read_byte(tile_data_address + (tile_data_line * 2 + 1), cart, joypad);

            let tile_data_byte_index = 7 - (window_x % 8);
            let tile_data_bit_1 = (tile_data_byte_1 >> tile_data_byte_index) & 1;
            let tile_data_bit_2 = (tile_data_byte_2 >> tile_data_byte_index) & 1;

            let color_index = (tile_data_bit_2 << 1) | tile_data_bit_1;
            let palette = mmu.read_byte(PPUMemory::BGP as u16, cart, joypad);

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

    pub fn draw_sprites_scanline(
        &mut self,
        scanline: u8,
        mmu: &mut MMU,
        cart: &mut Cart,
        joypad: &mut Joypad,
    ) {
        let lcdc = mmu.read_byte(PPUMemory::LCDC as u16, cart, joypad);
        let sprite_size_bit = (lcdc >> LCDCBits::ObjectSize as u8) & 1;
        let sprite_height: u8 = if sprite_size_bit == 0 { 8 } else { 16 };
        let mut visible_sprites: Vec<(i16, i16, u8, u8, u8)> = Vec::with_capacity(10);

        let oam_base: u16 = 0xFE00;

        // Scanline priority: OAM scan to hold 10 sprites
        for sprite_index in 0..40 {
            // each sprite is 4 bytes in OAM
            let oam_addr = oam_base + sprite_index * 4;

            let sprite_y = mmu.read_byte(oam_addr, cart, joypad) as i16 - 16;
            let sprite_x = mmu.read_byte(oam_addr + 1, cart, joypad) as i16 - 8;
            let tile_index = mmu.read_byte(oam_addr + 2, cart, joypad);
            let attributes = mmu.read_byte(oam_addr + 3, cart, joypad);

            if sprite_y <= (scanline as i16) && (scanline as i16) < sprite_y + sprite_height as i16
            {
                visible_sprites.push((
                    sprite_x,
                    sprite_y,
                    tile_index,
                    attributes,
                    sprite_index as u8,
                ));
                if visible_sprites.len() >= 10 {
                    break;
                }
            }
        }

        visible_sprites.sort_by(|a, b| {
            if a.0 != b.0 {
                b.0.cmp(&a.0)
            } else {
                // OAM index
                b.4.cmp(&a.4)
            }
        });

        // Draw sprites
        for (sprite_x, sprite_y, tile_index, attributes, _) in visible_sprites.into_iter() {
            let palette_select = (attributes >> OAMAttributesBits::PaletteNumber as u8) & 1;
            let x_flip = ((attributes >> OAMAttributesBits::XFlip as u8) & 1) != 0;
            let y_flip = ((attributes >> OAMAttributesBits::YFlip as u8) & 1) != 0;
            let background_priority = ((attributes >> OAMAttributesBits::Priority as u8) & 1) != 0;

            let actual_tile_index = if sprite_height == 16 {
                // For 8x16 sprites, this ensures the address lsb is 0 and the second tile index is 1
                // "0xNN & $FE, $0xNN | $01"
                tile_index & 0xFE
            } else {
                tile_index
            };

            let mut sprite_line = scanline as i16 - sprite_y;
            if y_flip {
                sprite_line = sprite_height as i16 - 1 - sprite_line;
            }

            let tile_for_line = if sprite_height == 16 && sprite_line >= 8 {
                actual_tile_index.wrapping_add(1)
            } else {
                actual_tile_index
            };

            // Sprites always use 0x8000 unsigned addressing
            let tile_data_base: u16 = 0x8000;
            let tile_data_address: u16 = tile_data_base + (tile_for_line as u16 * 16);

            let tile_data_line = (sprite_line as u16) % 8;

            let byte1 = mmu.read_byte(tile_data_address + tile_data_line * 2, cart, joypad);
            let byte2 = mmu.read_byte(tile_data_address + tile_data_line * 2 + 1, cart, joypad);

            for pixel in 0u8..8u8 {
                let bit_index_u8 = if x_flip { pixel } else { 7u8 - pixel };
                let shift = bit_index_u8 as u32;
                let bit1 = (byte1 >> shift) & 1;
                let bit2 = (byte2 >> shift) & 1;
                let color_index = (bit2 << 1) | bit1;

                // Color index 0 is transparent for sprites
                if color_index == 0 {
                    continue;
                }
                let palette =
                    mmu.read_byte(PPUMemory::OBP0 as u16 + palette_select as u16, cart, joypad);

                let color = match (palette >> (color_index * 2)) & 0b11 {
                    0 => COLOR_WHITE,
                    1 => COLOR_LIGHT_GRAY,
                    2 => COLOR_DARK_GRAY,
                    3 => COLOR_BLACK,
                    _ => COLOR_WHITE,
                };

                let px = sprite_x + pixel as i16;
                if px < 0 || px >= SCREEN_WIDTH as i16 {
                    continue;
                }

                let framebuffer_index = ((scanline as u32 * SCREEN_WIDTH) + px as u32) as usize;

                // If background priority is set, sprite is behind background except where background color is 0 (white)
                if background_priority {
                    let bg_color = self.framebuffer[framebuffer_index];
                    if bg_color != COLOR_WHITE {
                        // background has priority, skip drawing this pixel
                        continue;
                    }
                }

                self.framebuffer[framebuffer_index] = color;
            }
        }
    }
}
