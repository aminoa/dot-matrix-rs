use crate::mmu::MMU;
use std::rc::Rc;
use std::cell::RefCell;
pub struct CPU {
    pub A: u8,
    pub F: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,

    pub pc: u16,
    pub sp: u16,

    pub mmu: Rc<RefCell<MMU>>
}

impl CPU {
    pub fn new(mmu: Rc<RefCell<MMU>>) -> CPU {
        return CPU {
            A: 0x01,
            F: 0xB0,
            B: 0x00,
            C: 0x13,
            D: 0x00,
            E: 0xD8,
            H: 0x01,
            L: 0x4D,
            pc: 0x100,
            sp: 0xFFFE,
            mmu: mmu
        }
    }

    // helper functions; TODO: write a macro
    pub fn get_AF(&self) -> u16 {
        return (self.A as u16) << 8 | self.F as u16;
    }

    pub fn set_AF(&mut self, value: u16) {
        self.A = (value >> 8) as u8;
        self.F = value as u8;
    }

    pub fn get_BC(&self) -> u16 {
        return (self.B as u16) << 8 | self.C as u16;
    }

    pub fn set_BC(&mut self, value: u16) {
        self.B = (value >> 8) as u8;
        self.C = value as u8;
    }

    pub fn get_DE(&self) -> u16 {
        return (self.D as u16) << 8 | self.E as u16;
    }

    pub fn set_DE(&mut self, value: u16) {
        self.D = (value >> 8) as u8;
        self.E = value as u8;
    }

    pub fn get_HL(&self) -> u16 {
        return (self.H as u16) << 8 | self.L as u16;
    }

    pub fn set_HL(&mut self, value: u16) {
        self.H = (value >> 8) as u8;
        self.L = value as u8;
    }

    pub fn execute(&mut self, opcode: u8) {
        let arg_u8: u8 = self.mmu.borrow().read_byte(self.pc + 1);
        let arg_u16: u16 = self.mmu.borrow().read_short(self.pc + 1);

        match opcode {
            // 8 bit load instructions
            0x02 => self.mmu.borrow_mut().write_byte(self.get_BC(), self.A),
            0x06 => self.B = arg_u8,
            0x0A => self.A = self.mmu.borrow().read_byte(self.get_BC()),
            0x0E => self.C = arg_u8,
            0x12 => self.mmu.borrow_mut().write_byte(self.get_DE(), self.A),
            0x16 => self.D = arg_u8,
            0x1A => self.A = self.mmu.borrow().read_byte(self.get_DE()),
            0x1E => self.E = arg_u8,
            0x22 => {
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.A);
                self.set_HL(self.get_HL() + 1);
            },
            0x26 => self.H = arg_u8,
            0x2A => {
                self.A = self.mmu.borrow().read_byte(self.get_HL());
                self.set_HL(self.get_HL() + 1);
            },
            0x2E => self.L = arg_u8,
            0x32 => {
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.A);
                self.set_HL(self.get_HL() + 1);
            },
            0x36 => self.mmu.borrow_mut().write_byte(self.get_HL(), arg_u8),
            0x3A => {
                self.A = self.mmu.borrow().read_byte(self.get_HL());
                self.set_HL(self.get_HL() - 1);
            },
            0x3E => self.A = arg_u8,

            0x40 => self.B = self.B,
            0x41 => self.B = self.C,
            0x42 => self.B = self.D,
            0x43 => self.B = self.E,
            0x44 => self.B = self.H,
            0x45 => self.B = self.L,
            0x46 => self.B = self.mmu.borrow().read_byte(self.get_HL()),
            0x47 => self.B = self.A,
            0x48 => self.C = self.B,
            0x49 => self.C = self.C,
            0x4A => self.C = self.D,
            0x4B => self.C = self.E,
            0x4C => self.C = self.H,
            0x4D => self.C = self.L,
            0x4E => self.C = self.mmu.borrow().read_byte(self.get_HL()),
            0x4F => self.C = self.A,
            0x50 => self.D = self.B,
            0x51 => self.D = self.C,
            0x52 => self.D = self.D,
            0x53 => self.D = self.E,
            0x54 => self.D = self.H,
            0x55 => self.D = self.L,
            0x56 => self.D = self.mmu.borrow().read_byte(self.get_HL()),
            0x57 => self.D = self.A,
            0x58 => self.E = self.B,
            0x59 => self.E = self.C,
            0x5A => self.E = self.D,
            0x5B => self.E = self.E,
            0x5C => self.E = self.H,
            0x5D => self.E = self.L,
            0x5E => self.E = self.mmu.borrow().read_byte(self.get_HL()),
            0x5F => self.E = self.A,
            0x60 => self.H = self.B,
            0x61 => self.H = self.C,
            0x62 => self.H = self.D,
            0x63 => self.H = self.E,
            0x64 => self.H = self.H,
            0x65 => self.H = self.L,
            0x66 => self.H = self.mmu.borrow().read_byte(self.get_HL()),
            0x67 => self.H = self.A,
            0x68 => self.L = self.B,
            0x69 => self.L = self.C,
            0x6A => self.L = self.D,
            0x6B => self.L = self.E,
            0x6C => self.L = self.H,
            0x6D => self.L = self.L,
            0x6E => self.L = self.mmu.borrow().read_byte(self.get_HL()),
            0x6F => self.L = self.A,
            0x70 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.B),
            0x71 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.C),
            0x72 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.D),
            0x73 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.E),
            0x74 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.H),
            0x75 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.L),
            0x77 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.A),
            0x78 => self.A = self.B,
            0x79 => self.A = self.C,
            0x7A => self.A = self.D,
            0x7B => self.A = self.E,
            0x7C => self.A = self.H,
            0x7D => self.A = self.L,
            0x7E => self.A = self.mmu.borrow().read_byte(self.get_HL()),
            0x7F => self.A = self.A,

            0xE0 => self.mmu.borrow_mut().write_byte(0xFF00 + arg_u8 as u16, self.A),
            0xE2 => self.mmu.borrow_mut().write_byte(0xFF00 + self.C as u16, self.A),

            0xEA => self.mmu.borrow_mut().write_byte(arg_u16, self.A),
            0xF0 => self.A = self.mmu.borrow().read_byte(0xFF00 + arg_u8 as u16),
            0xF2 => self.A = self.mmu.borrow().read_byte(0xFF00 + self.C as u16),
            0xFA => self.A = self.mmu.borrow().read_byte(arg_u16),

            // 16 bit load instructions

            _ => println!("Opcode not implemented: {:X}", opcode)
        }
        self.pc += 4;
    }
}