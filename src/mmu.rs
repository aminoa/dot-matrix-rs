pub struct MMU {
    pub ram: Vec<u8>,
    pub cart: Vec<u8>,
}

// 16 bit address bus, but not 65,536 valid addresses
// a 64kb RAM array isn't accurate (https://gbdev.io/pandocs/Memory_Map.html)

impl MMU {
    pub fn new(cart: Vec<u8>) -> MMU {
        let mut ram = vec![0; 0x10000];
        ram[0xFF00] = 0xF;

        return MMU {
            ram: ram,
            cart: cart
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // lock LCD for blargs tests
            0x0..0x7FFF => self.cart[addr as usize],
            _ => self.ram[addr as usize]

        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // DIV writes should be trapped
            0xFF04 => self.ram[addr as usize] = 0,
            0xFF00 => (),
            0x0..0x7FFF => self.cart[addr as usize] = value,
            _ => self.ram[addr as usize] = value
        }
    }

    pub fn read_short(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | (self.read_byte(addr + 1) as u16) << 8
    }

    pub fn write_short(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }
}