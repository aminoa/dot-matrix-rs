pub struct CPU {
    // Registers
    AF: u16,
    BC: u16,
    DE: u16,
    HL: u16,

    pc: u16,
    sp: u16
}

impl CPU {
    pub fn new() -> CPU {
        return CPU {
            AF: 0x01B0,
            BC: 0x0013,
            DE: 0x00D8,
            HL: 0x014D,
        
            pc: 0x100,
            sp: 0xFFFE
        }
    }
}