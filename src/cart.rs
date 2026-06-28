use crate::consts::{RAM_BANK_SIZE, RAM_START_ADDR, ROM_BANK_SIZE};

// R is RAM, B is Battery
enum MBC {
    None,
    MBC1,
    MBC3,
}

pub struct Cart {
    pub rom: Vec<u8>,
    pub title: String,
    pub cartridge_type: u8,
    pub rom_size_code: u8,
    pub rom_size_bytes: usize,
    pub ram_size_code: u8,
    pub ram_size_bytes: usize,
    pub ram_enabled: bool,
    pub rom_bank_selected: u8,
    pub ram_bank_selected: u8,
    pub cartridge_type_mbc: MBC,
    pub ram: Vec<u8>,
    pub banking_mode: bool, // ranges locked to bank 0 by default
}

impl Cart {
    pub fn from_rom(rom: Vec<u8>) -> Cart {
        let title_bytes = &rom[0x134..0x144];
        let title =
            String::from_utf8_lossy(title_bytes.iter().cloned().collect::<Vec<u8>>().as_slice())
                .trim_end_matches('\0')
                .to_string();
        let cartridge_type = rom[0x147];
        let cartridge_type_mbc = match cartridge_type {
            0x0 => MBC::None,
            0x1 | 0x2 | 0x3 => MBC::MBC1,
            0x11 | 0x12 | 0x13 => MBC::MBC3,
            _ => MBC::None,
        };

        let rom_size_code = rom[0x148];
        let ram_size_code = rom[0x149];

        let rom_size_bytes = match rom_size_code {
            0x00 => 32 * 1024,
            0x01 => 64 * 1024,
            0x02 => 128 * 1024,
            0x03 => 256 * 1024,
            0x04 => 512 * 1024,
            0x05 => 1 * 1024 * 1024,
            0x06 => 2 * 1024 * 1024,
            0x07 => 4 * 1024 * 1024,
            _ => panic!("Unsupported ROM size code: {}", rom_size_code),
        };

        let ram_size_bytes = match ram_size_code {
            0x00 => 0,
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            _ => panic!("Unsupported RAM size code: {}", ram_size_code),
        };

        let ram = vec![0u8; ram_size_bytes as usize];

        Cart {
            rom,
            title,
            cartridge_type,
            rom_size_code,
            rom_size_bytes,
            ram_size_code,
            ram_size_bytes,
            ram_enabled: false,
            ram: ram,
            rom_bank_selected: 1,
            cartridge_type_mbc: cartridge_type_mbc,
            ram_bank_selected: 0,
            banking_mode: true,
        }
    }

    pub fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                let banked_addr = (self.rom_bank_selected as usize * ROM_BANK_SIZE as usize)
                    + (addr as usize - ROM_BANK_SIZE as usize);
                self.rom[banked_addr as usize]
            }
            _ => panic!("Address out of ROM range: {:04X}", addr),
        }
    }

    pub fn write_rom(&mut self, addr: u16, val: u8) {
        match self.cartridge_type_mbc {
            MBC::None => (),
            MBC::MBC1 => match addr {
                0x0000..0x2000 => self.ram_enabled = val == 0x0A,
                0x2000..0x4000 => self.select_rom_bank(val),
                0x4000..0x6000 => {
                    let reg = (val & 0x3);
                    if self.banking_mode && self.ram_size_bytes >= 32 * 1024 {
                        // min 32 KiB
                        self.ram_bank_selected = reg;
                    } else if self.rom_size_bytes >= 1 * 1024 * 1024 {
                        // min 1 MiB
                        self.rom_bank_selected |= reg << 5;
                    }
                }
                0x6000..0x7FFF => {
                    let reg = val & 0x1;
                    self.banking_mode = reg == 0;
                }
                _ => panic!("Address out of ROM range: {:04X}", addr),
            },
            _ => panic!("Error: Unrecognized MBC"),
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        let banked_addr = (addr - RAM_START_ADDR) + (self.ram_bank_selected as u16 * RAM_BANK_SIZE);
        return self.ram[banked_addr as usize];
    }

    pub fn select_rom_bank(&mut self, value: u8) {
        let bank = value & 0x1F; // 5 bit register
        match bank {
            0 => self.rom_bank_selected = 1,
            0x20 | 0x40 | 0x60 => self.rom_bank_selected = bank + 1,
            _ => self.rom_bank_selected = bank,
        }
    }
}
