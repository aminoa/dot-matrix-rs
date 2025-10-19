pub struct Cart {
    pub rom: Vec<u8>,
    pub title: String,
    pub cartridge_type: u8,
    pub rom_size_code: u8,
    pub rom_size_bytes: usize,
    pub ram_size_code: u8,
    pub ram_size_bytes: usize,
}

impl Cart {
    pub fn from_rom(rom: Vec<u8>) -> Cart {
        let title_bytes = &rom[0x134..0x144];
        let title = String::from_utf8(title_bytes.iter().cloned().collect()).unwrap();
        let cartridge_type = rom[0x147];
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

        Cart {
            rom,
            title,
            cartridge_type,
            rom_size_code,
            rom_size_bytes,
            ram_size_code,
            ram_size_bytes,
        }
    }
}
