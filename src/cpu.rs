use crate::mmu::MMU;
use std::rc::Rc;
use std::cell::RefCell;

enum FlagRegister {
    Zero = 7,
    Sub = 6,
    HalfCarry = 5,
    Carry = 4
}

pub struct CPU {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,

    pub ime: bool,
    pub stopped: bool,
    pub halted: bool,

    pub mmu: Rc<RefCell<MMU>>
}

impl CPU {
    pub fn new(mmu: Rc<RefCell<MMU>>) -> CPU {
        return CPU {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,

            pc: 0x100,
            sp: 0xFFFE,

            ime: false,
            stopped: false,
            halted: false,

            mmu: mmu
        }
    }

    pub fn get_flag(&self, flag: FlagRegister) -> u8 {
        return self.f & (1 << flag as u8);
    }

    pub fn set_flag(&mut self, flag: FlagRegister, value: bool) {
        if value {
            self.f |= 1 << flag as u8;
        } else {
            self.f &= !(1 << flag as u8); 
        }
    }

    // helper functions; TODO: write a macro
    pub fn get_AF(&self) -> u16 {
        return (self.a as u16) << 8 | self.f as u16;
    }

    pub fn set_AF(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }

    pub fn get_BC(&self) -> u16 {
        return (self.b as u16) << 8 | self.c as u16;
    }

    pub fn set_BC(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn get_DE(&self) -> u16 {
        return (self.d as u16) << 8 | self.e as u16;
    }

    pub fn set_DE(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn get_HL(&self) -> u16 {
        return (self.h as u16) << 8 | self.l as u16;
    }

    pub fn set_HL(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    pub fn pop(&mut self) -> u16 {
        let val = self.mmu.borrow().read_short(self.sp);
        self.sp += 2;
        return val;
    }

    pub fn push(&mut self, value: u16) {
        self.sp -= 2;
        self.mmu.borrow_mut().write_short(self.sp, value);
    }

    pub fn inc(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_add(1);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, (result & 0x0F) == 0x00);

        return result;
    }

    pub fn dec(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_sub(1);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, true);
        self.set_flag(FlagRegister::HalfCarry, (result & 0x0F) == 0x0F);

        return result;
    }

    pub fn rlca(&mut self) {
        let carry = self.a >> 7;
        self.a = (self.a << 1) | carry;

        self.set_flag(FlagRegister::Zero, false);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);
    }

    pub fn rrca(&mut self) {
        let carry = self.a & 1;
        self.a = (self.a >> 1) | (carry << 7);

        self.set_flag(FlagRegister::Zero, false);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);
    }

    pub fn rla(&mut self) {
        let carry = self.a >> 7;
        self.a = (self.a << 1) | self.get_flag(FlagRegister::Carry);

        self.set_flag(FlagRegister::Zero, false);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);
    }

    pub fn rra(&mut self) {
        let carry = self.a & 1;
        self.a = (self.a >> 1) | (self.get_flag(FlagRegister::Carry) << 7);

        self.set_flag(FlagRegister::Zero, false);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);
    }

    pub fn rlc(&mut self, reg: u8) -> u8 {
        let carry = reg >> 7;
        let result = (reg << 1) | carry;

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn rrc(&mut self, reg: u8) -> u8 {
        let carry = reg & 1;
        let result = (reg >> 1) | (carry << 7);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn rl(&mut self, reg: u8) -> u8 {
        let carry = reg >> 7;
        let result = (reg << 1) | self.get_flag(FlagRegister::Carry);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn rr(&mut self, reg: u8) -> u8 {
        let carry = reg & 1;
        let result = (reg >> 1) | (self.get_flag(FlagRegister::Carry) << 7);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn sla(&mut self, reg: u8) -> u8 {
        let carry = reg >> 7;
        let result = reg << 1;

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn sra(&mut self, reg: u8) -> u8 {
        let carry = reg & 1;
        let result = (reg >> 1) | (reg & 0x80);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn swap(&mut self, reg: u8) -> u8 {
        let result = (reg >> 4) | (reg << 4);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, false);

        return result;
    }

    pub fn add_a(&mut self, value: u8) {
        let result = self.a.wrapping_add(value);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, (self.a & 0x0F) + (value & 0x0F) > 0x0F);
        self.set_flag(FlagRegister::Carry, (self.a as u16) + (value as u16) > 0xFF);

        self.a = result;
    }

    pub fn add_hl(&mut self, value: u16) {
        let result = self.get_HL().wrapping_add(value);

        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, (self.get_HL() & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.set_flag(FlagRegister::Carry, (self.get_HL() as u32) + (value as u32) > 0xFFFF);

        self.set_HL(result);
    }


    // adds to A with carry
    pub fn adc(&mut self, value: u8) {
        let carry = self.get_flag(FlagRegister::Carry);
        let result = self.a.wrapping_add(value).wrapping_add(carry);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, (self.a & 0x0F) + (value & 0x0F) + carry > 0x0F);
        self.set_flag(FlagRegister::Carry, (self.a as u16) + (value as u16) + (carry as u16) > 0xFF);

        self.a = result;
    }

    // subs to A
    pub fn sub(&mut self, value: u8) {
        let result = self.a.wrapping_sub(value);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, true);
        self.set_flag(FlagRegister::HalfCarry, (self.a & 0x0F) < (value & 0x0F));
        self.set_flag(FlagRegister::Carry, (self.a as u16) < (value as u16));

        self.a = result;
    }

    // adds to sp
    pub fn add_signed(&mut self, value: i8) {
        let result = self.sp.wrapping_add(value as u16);

        self.set_flag(FlagRegister::Zero, false);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, (self.sp & 0x0F) + (value as u16 & 0x0F) > 0x0F);
        self.set_flag(FlagRegister::Carry, (self.sp & 0xFF) + (value as u16 & 0xFF) > 0xFF);

        self.sp = result;
    }

    pub fn sbc(&mut self, value: u8) {
        let carry = self.get_flag(FlagRegister::Carry);
        let result = self.a.wrapping_sub(value).wrapping_sub(carry);

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, true);
        self.set_flag(FlagRegister::HalfCarry, (self.a & 0x0F) < (value & 0x0F) + carry);
        self.set_flag(FlagRegister::Carry, (self.a as u16) < (value as u16) + (carry as u16));

        self.a = result;
    }

    pub fn srl(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = value >> 1;

        self.set_flag(FlagRegister::Zero, result == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, carry == 1);

        return result;
    }

    pub fn bit(&mut self, bit: u8, value: u8) {
        self.set_flag(FlagRegister::Zero, (value & (1 << bit)) == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, true);
    }

    pub fn res(&mut self, bit: u8, value: u8) -> u8 {
        return value & !(1 << bit);
    }

    pub fn set(&mut self, bit: u8, value: u8) -> u8 {
        return value | (1 << bit);
    }

    pub fn and(&mut self, value: u8) {
        self.a &= value;

        self.set_flag(FlagRegister::Zero, self.a == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, true);
        self.set_flag(FlagRegister::Carry, false);
    }

    pub fn xor(&mut self, value: u8) {
        self.a ^= value;

        self.set_flag(FlagRegister::Zero, self.a == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, false);
    }

    pub fn or(&mut self, value: u8) {
        self.a |= value;

        self.set_flag(FlagRegister::Zero, self.a == 0);
        self.set_flag(FlagRegister::Sub, false);
        self.set_flag(FlagRegister::HalfCarry, false);
        self.set_flag(FlagRegister::Carry, false);
    }

    pub fn cp(&mut self, value: u8) {
        self.set_flag(FlagRegister::Zero, self.a == value);
        self.set_flag(FlagRegister::Sub, true);
        self.set_flag(FlagRegister::HalfCarry, (self.a & 0x0F) < (value & 0x0F));
        self.set_flag(FlagRegister::Carry, (self.a as u16) < (value as u16));
    }

    // Credit: https://blog.ollien.com/posts/gb-daa/
    pub fn daa(&mut self) -> u8 {
        let mut offset: u8 = 0;

        let half_carry = self.get_flag(FlagRegister::HalfCarry);
        let carry = self.get_flag(FlagRegister::Carry);
        let subtract = self.get_flag(FlagRegister::Sub);
    
        if (subtract == 0 && self.a & 0xF > 0x09) || half_carry == 1 {
            offset |= 0x06;
        }
    
        if (subtract == 0 && self.a > 0x99) || carry == 1 {
            offset |= 0x60;
        }
    
        return if subtract == 0 {
            self.a.wrapping_add(offset)
        } else {
            self.a.wrapping_sub(offset)
        };
    }

    pub fn execute(&mut self, opcode: u8) {
        let arg_u8: u8 = self.mmu.borrow().read_byte(self.pc + 1);
        let arg_u16: u16 = self.mmu.borrow().read_short(self.pc + 1);


        match opcode {
            // 8 bit load instructions
            0x02 => self.mmu.borrow_mut().write_byte(self.get_BC(), self.a),
            0x06 => self.b = arg_u8,
            0x0A => self.a = self.mmu.borrow().read_byte(self.get_BC()),
            0x0E => self.c = arg_u8,
            0x12 => self.mmu.borrow_mut().write_byte(self.get_DE(), self.a),
            0x16 => self.d = arg_u8,
            0x1A => self.a = self.mmu.borrow().read_byte(self.get_DE()),
            0x1E => self.e = arg_u8,
            0x22 => {
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.a);
                self.set_HL(self.get_HL() + 1);
            },
            0x26 => self.h = arg_u8,
            0x2A => {
                self.a = self.mmu.borrow().read_byte(self.get_HL());
                self.set_HL(self.get_HL() + 1);
            },
            0x2E => self.l = arg_u8,
            0x32 => {
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.a);
                self.set_HL(self.get_HL() + 1);
            },
            0x36 => self.mmu.borrow_mut().write_byte(self.get_HL(), arg_u8),
            0x3A => {
                self.a = self.mmu.borrow().read_byte(self.get_HL());
                self.set_HL(self.get_HL() - 1);
            },
            0x3E => self.a = arg_u8,

            0x40 => self.b = self.b,
            0x41 => self.b = self.c,
            0x42 => self.b = self.d,
            0x43 => self.b = self.e,
            0x44 => self.b = self.h,
            0x45 => self.b = self.l,
            0x46 => self.b = self.mmu.borrow().read_byte(self.get_HL()),
            0x47 => self.b = self.a,
            0x48 => self.c = self.b,
            0x49 => self.c = self.c,
            0x4A => self.c = self.d,
            0x4B => self.c = self.e,
            0x4C => self.c = self.h,
            0x4D => self.c = self.l,
            0x4E => self.c = self.mmu.borrow().read_byte(self.get_HL()),
            0x4F => self.c = self.a,
            0x50 => self.d = self.b,
            0x51 => self.d = self.c,
            0x52 => self.d = self.d,
            0x53 => self.d = self.e,
            0x54 => self.d = self.h,
            0x55 => self.d = self.l,
            0x56 => self.d = self.mmu.borrow().read_byte(self.get_HL()),
            0x57 => self.d = self.a,
            0x58 => self.e = self.b,
            0x59 => self.e = self.c,
            0x5A => self.e = self.d,
            0x5B => self.e = self.e,
            0x5C => self.e = self.h,
            0x5D => self.e = self.l,
            0x5E => self.e = self.mmu.borrow().read_byte(self.get_HL()),
            0x5F => self.e = self.a,
            0x60 => self.h = self.b,
            0x61 => self.h = self.c,
            0x62 => self.h = self.d,
            0x63 => self.h = self.e,
            0x64 => self.h = self.h,
            0x65 => self.h = self.l,
            0x66 => self.h = self.mmu.borrow().read_byte(self.get_HL()),
            0x67 => self.h = self.a,
            0x68 => self.l = self.b,
            0x69 => self.l = self.c,
            0x6A => self.l = self.d,
            0x6B => self.l = self.e,
            0x6C => self.l = self.h,
            0x6D => self.l = self.l,
            0x6E => self.l = self.mmu.borrow().read_byte(self.get_HL()),
            0x6F => self.l = self.a,
            0x70 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.b),
            0x71 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.c),
            0x72 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.d),
            0x73 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.e),
            0x74 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.h),
            0x75 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.l),
            0x77 => self.mmu.borrow_mut().write_byte(self.get_HL(), self.a),
            0x78 => self.a = self.b,
            0x79 => self.a = self.c,
            0x7A => self.a = self.d,
            0x7B => self.a = self.e,
            0x7C => self.a = self.h,
            0x7D => self.a = self.l,
            0x7E => self.a = self.mmu.borrow().read_byte(self.get_HL()),
            0x7F => self.a = self.a,

            0xE0 => self.mmu.borrow_mut().write_byte(0xFF00 + arg_u8 as u16, self.a),
            0xE2 => self.mmu.borrow_mut().write_byte(0xFF00 + self.c as u16, self.a),

            0xEA => self.mmu.borrow_mut().write_byte(arg_u16, self.a),
            0xF0 => self.a = self.mmu.borrow().read_byte(0xFF00 + arg_u8 as u16),
            0xF2 => self.a = self.mmu.borrow().read_byte(0xFF00 + self.c as u16),
            0xFA => self.a = self.mmu.borrow().read_byte(arg_u16),

            // 16 bit load instructions
            0x01 => self.set_BC(arg_u16),
            0x08 => self.mmu.borrow_mut().write_short(arg_u16, self.sp),
            0x11 => self.set_DE(arg_u16),
            0x21 => self.set_HL(arg_u16),
            0x31 => self.sp = arg_u16,
            0xC1 => {
                let temp = self.pop();
                self.set_BC(temp);
            }
            0xD1 => {
                let temp = self.pop();
                self.set_DE(temp);
            }
            0xD5 => self.push(self.get_DE()),
            0xE1 => {
                let temp = self.pop();
                self.set_HL(temp);
            },
            0xE5 => self.push(self.get_HL()),
            0xF1 => {
                let temp = self.pop() & 0xFFF0;
                self.set_AF(temp);
            },
            0xF5 => self.push(self.get_AF()),
            0xF8 => {
                let temp = self.sp + (arg_u8 as i8) as u16;
                self.set_HL(temp);

                self.set_flag(FlagRegister::Zero, false);
                self.set_flag(FlagRegister::Sub, false);
                self.set_flag(FlagRegister::HalfCarry, ((self.sp & 0x0F) + (arg_u8 as u16 & 0xFF)) > 0x0F);
                self.set_flag(FlagRegister::Carry, (self.sp & 0xFF) + (arg_u8 as u16 & 0xFF) > 0xFF);
            },
            0xF9 => self.sp = self.get_HL(),

            // 8 bit arithmetic/logical instructions
            0x04 => self.b = self.inc(self.b),
            0x05 => self.b = self.dec(self.b),
            0x0C => self.c = self.inc(self.c),
            0x0D => self.c = self.dec(self.c),
            0x14 => self.d = self.inc(self.d),
            0x15 => self.d = self.dec(self.d),
            0x1C => self.e = self.inc(self.e),
            0x1D => self.e = self.dec(self.e),
            0x24 => self.h = self.inc(self.h),
            0x25 => self.h = self.dec(self.h),
            0x2C => self.l = self.inc(self.l),
            0x2D => self.l = self.dec(self.l),
            0x34 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let temp = self.inc(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), temp);
            },
            0x35 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let temp = self.dec(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), temp);
            },
            0x3C => self.a = self.inc(self.a),
            0x3D => self.a = self.dec(self.a),

            0x80 => self.add_a(self.b),
            0x81 => self.add_a(self.c),
            0x82 => self.add_a(self.d),
            0x83 => self.add_a(self.e),
            0x84 => self.add_a(self.h),
            0x85 => self.add_a(self.l),
            0x86 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.add_a(temp);
            },
            0x87 => self.add_a(self.a),
            0x88 => self.adc(self.b),
            0x89 => self.adc(self.c),
            0x8A => self.adc(self.d),
            0x8B => self.adc(self.e),
            0x8C => self.adc(self.h),
            0x8D => self.adc(self.l),
            0x8E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.adc(temp);
            },
            0x8F => self.adc(self.a),
            0x90 => self.sub(self.b),
            0x91 => self.sub(self.c),
            0x92 => self.sub(self.d),
            0x93 => self.sub(self.e),
            0x94 => self.sub(self.h),
            0x95 => self.sub(self.l),
            0x96 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.sub(temp);
            },
            0x97 => self.sub(self.a),
            0x98 => self.sbc(self.b),
            0x99 => self.sbc(self.c),
            0x9A => self.sbc(self.d),
            0x9B => self.sbc(self.e),
            0x9C => self.sbc(self.h),
            0x9D => self.sbc(self.l),
            0x9E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.sbc(temp);
            },
            0x9F => self.sbc(self.a),
            0xA0 => self.and(self.b),
            0xA1 => self.and(self.c),
            0xA2 => self.and(self.d),
            0xA3 => self.and(self.e),
            0xA4 => self.and(self.h),
            0xA5 => self.and(self.l),
            0xA6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.and(temp);
            },
            0xA7 => self.and(self.a),
            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.xor(temp);
            },
            0xAF => self.xor(self.a),
            0xB0 => self.or(self.b),
            0xB1 => self.or(self.c),
            0xB2 => self.or(self.d),
            0xB3 => self.or(self.e),
            0xB4 => self.or(self.h),
            0xB5 => self.or(self.l),
            0xB6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.or(temp);
            },
            0xB7 => self.or(self.a),
            0xB8 => self.cp(self.b),
            0xB9 => self.cp(self.c),
            0xBA => self.cp(self.d),
            0xBB => self.cp(self.e),
            0xBC => self.cp(self.h),
            0xBD => self.cp(self.l),
            0xBE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.cp(temp);
            },
            0xBF => self.cp(self.a),

            0xC6 => self.add_a(arg_u8),
            0xCE => self.adc(arg_u8),
            0xD6 => self.sub(arg_u8),
            0xDE => self.sbc(arg_u8),
            0xE6 => self.and(arg_u8),
            0xEE => self.xor(arg_u8),
            0xF6 => self.or(arg_u8),
            0xFE => self.cp(arg_u8),

            // 16-bit arithmetic/logic instructions
            0x03 => self.set_BC(self.get_BC() + 1),
            0x09 => self.add_hl(self.get_BC()),
            0x0B => self.set_BC(self.get_BC() - 1),
            0x13 => self.set_DE(self.get_DE() + 1),
            0x19 => self.add_hl(self.get_DE()),
            0x1B => self.set_DE(self.get_DE() - 1),
            0x23 => self.set_HL(self.get_HL() + 1),
            0x29 => self.add_hl(self.get_HL()),
            0x2B => self.set_HL(self.get_HL() - 1),
            0x33 => self.sp += 1,
            0x39 => self.add_hl(self.sp),
            0x3B => self.sp -= 1,
            0xE8 => self.add_signed(arg_u8 as i8),

            // 8-bit shift, rotate, and bit instructions
            0x07 => self.rlca(),
            0x0F => self.rrca(),
            0x17 => self.rla(),
            0x1F => self.rra(),
            0xCB => println!("CB prefix not implemented"),

            // CPU control instructions
            0x00 => (),
            0x10 => self.stopped = true, 
            0x27 => self.a = self.daa(),
            0x37 => {
                self.set_flag(FlagRegister::Sub, false);
                self.set_flag(FlagRegister::HalfCarry, false);
                self.set_flag(FlagRegister::Carry, true);
            },
            0x3F => {
                let carry = self.get_flag(FlagRegister::Carry) == 1;
                self.set_flag(FlagRegister::Sub, false);
                self.set_flag(FlagRegister::HalfCarry, false);
                self.set_flag(FlagRegister::Carry, !carry);
            },
            0x76 => self.halted = true,
            0xF3 => self.ime = false,
            0xFB => self.ime = true,

            0x18 => self.pc = self.pc.wrapping_add(arg_u8 as u16),
            0x20 => {
                if self.get_flag(FlagRegister::Zero) == 0 { 
                    self.pc = self.pc.wrapping_add(arg_u8 as u16);
                } 
            },
            0x28 => {
                if self.get_flag(FlagRegister::Zero) == 1 { 
                    self.pc = self.pc.wrapping_add(arg_u8 as u16);
                } 
            },
            0x30 => {
                if self.get_flag(FlagRegister::Carry) == 0 { 
                    self.pc = self.pc.wrapping_add(arg_u8 as u16);
                } 
            },
            0x38 => {
                if self.get_flag(FlagRegister::Carry) == 1 { 
                    self.pc = self.pc.wrapping_add(arg_u8 as u16);
                } 
            },
            0xC0 => {
                if self.get_flag(FlagRegister::Zero) == 0 {
                    self.pc = self.pop();
                }
            },
            0xC2 => {
                if self.get_flag(FlagRegister::Zero) == 0 {
                    self.pc = arg_u16;
                }
            },
            0xC3 => self.pc = arg_u16,
            0xC4 => {
                if self.get_flag(FlagRegister::Zero) == 0 {
                    self.push(self.pc + 2);
                    self.pc = arg_u16;
                }
            },
            0xC7 => {
                self.push(self.pc);
                self.pc = 0x00;
            },
            0xC8 => {
                if self.get_flag(FlagRegister::Zero) == 1 {
                    self.pc = self.pop();
                }
            },
            0xC9 => self.pc = self.pop(),
            0xCA => {
                if self.get_flag(FlagRegister::Zero) == 1 {
                    self.pc = arg_u16;
                }
            },
            0xCC => {
                if self.get_flag(FlagRegister::Zero) == 1 {
                    self.push(self.pc + 2);
                    self.pc = arg_u16;
                }
            },
            0xCD => {
                self.push(self.pc + 2);
                self.pc = arg_u16;
            },
            0xD0 => {
                if self.get_flag(FlagRegister::Carry) == 0 {
                    self.pc = self.pop();
                }
            },
            0xD2 => {
                if self.get_flag(FlagRegister::Carry) == 0 {
                    self.pc = arg_u16;
                }
            },
            0xD4 => {
                if self.get_flag(FlagRegister::Carry) == 0 {
                    self.push(self.pc + 2);
                    self.pc = arg_u16;
                }
            },
            0xD7 => {
                self.push(self.pc);
                self.pc = 0x10;
            },
            0xD8 => {
                if self.get_flag(FlagRegister::Carry) == 1 {
                    self.pc = self.pop();
                }
            },
            0xD9 => {
                self.pc = self.pop();
                self.ime = true;
            },

            0xDA => {
                if self.get_flag(FlagRegister::Carry) == 1 {
                    self.pc = arg_u16;
                }
            },
            0xDC => {
                if self.get_flag(FlagRegister::Carry) == 1 {
                    self.push(self.pc + 2);
                    self.pc = arg_u16;
                }
            },
            0xDF => {
                self.push(self.pc);
                self.pc = 0x18;
            },
            0xE7 => {
                self.push(self.pc);
                self.pc = 0x20;
            },
            0xE9 => self.pc = self.get_HL(),
            0xEF => {
                self.push(self.pc);
                self.pc = 0x28;
            },
            0xF7 => {
                self.push(self.pc);
                self.pc = 0x30;
            },
            0xFF => {
                self.push(self.pc);
                self.pc = 0x38;
            },

            _ => println!("Error: Opcode unknown: {:X}; something has gone seriously wrong", opcode)
        }
    }
}