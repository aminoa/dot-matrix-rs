pub struct MMU {
    pub ram: Vec<u8>,
    pub cart: Vec<u8>
}

impl MMU {
    pub fn new(cart: Vec<u8>) -> MMU {
        let ram = Vec::new();
        return MMU {
            ram: ram,
            cart: cart
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0..0x7FFF => self.cart[addr as usize],
            _ => 0xFF
        }
    }
}