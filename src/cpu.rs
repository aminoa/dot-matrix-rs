pub struct CPU {
    // Registers
    pub AF: u16,
    pub BC: u16,
    pub DE: u16,
    pub HL: u16,

    pub pc: u16,
    pub sp: u16
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

    pub fn execute(&mut self, opcode: u8) {
        // increment pc by opcode const
        match opcode {
            // 0x02 => mmu.write_byte(BC, A),
            // 0x06 => mmu.write_byte(),
        }
        self.pc += 4;
    }
}