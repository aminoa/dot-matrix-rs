use crate::mmu::MMU;
use std::rc::Rc;
use std::cell::RefCell;

use crate::consts::{OPCODES, CB_OPCODES};

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

    pub fn res(&self, bit: u8, value: u8) -> u8 {
        return value & !(1 << bit);
    }

    pub fn set(&self, bit: u8, value: u8) -> u8 {
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

        if opcode == 0xCB { 
            self.pc += CB_OPCODES[arg_u8 as usize].bytes as u16;
        } else {
            self.pc += OPCODES[opcode as usize].bytes as u16;
        }

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
            0xCB => self.execute_cb(arg_u8),

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
            
            _ => println!("Error: Opcode unknown: {:X}; something has gone seriously wrong", opcode),
        }
    }

    pub fn execute_cb(&mut self, opcode: u8) {
        match opcode {
            0x00 => self.b = self.rlc(self.b),
            0x01 => self.c = self.rlc(self.c),
            0x02 => self.d = self.rlc(self.d),
            0x03 => self.e = self.rlc(self.e),
            0x04 => self.h = self.rlc(self.h),
            0x05 => self.l = self.rlc(self.l),
            0x06 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.rlc(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x07 => self.a = self.rlc(self.a),
            0x08 => self.b = self.rrc(self.b),
            0x09 => self.c = self.rrc(self.c),
            0x0A => self.d = self.rrc(self.d),
            0x0B => self.e = self.rrc(self.e),
            0x0C => self.h = self.rrc(self.h),
            0x0D => self.l = self.rrc(self.l),
            0x0E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.rrc(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x0F => self.a = self.rrc(self.a),

            0x10 => self.b = self.rl(self.b),
            0x11 => self.c = self.rl(self.c),
            0x12 => self.d = self.rl(self.d),
            0x13 => self.e = self.rl(self.e),
            0x14 => self.h = self.rl(self.h),
            0x15 => self.l = self.rl(self.l),
            0x16 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.rl(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x17 => self.a = self.rl(self.a),
            0x18 => self.b = self.rr(self.b),
            0x19 => self.c = self.rr(self.c),
            0x1A => self.d = self.rr(self.d),
            0x1B => self.e = self.rr(self.e),
            0x1C => self.h = self.rr(self.h),
            0x1D => self.l = self.rr(self.l),
            0x1E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.rr(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x1F => self.a = self.rr(self.a),

            0x20 => self.b = self.sla(self.b),
            0x21 => self.c = self.sla(self.c),
            0x22 => self.d = self.sla(self.d),
            0x23 => self.e = self.sla(self.e),
            0x24 => self.h = self.sla(self.h),
            0x25 => self.l = self.sla(self.l),
            0x26 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.sla(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x27 => self.a = self.sla(self.a),
            0x28 => self.b = self.sra(self.b),
            0x29 => self.c = self.sra(self.c),
            0x2A => self.d = self.sra(self.d),
            0x2B => self.e = self.sra(self.e),
            0x2C => self.h = self.sra(self.h),
            0x2D => self.l = self.sra(self.l),
            0x2E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.sra(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x2F => self.a = self.sra(self.a),

            0x30 => self.b = self.swap(self.b),
            0x31 => self.c = self.swap(self.c),
            0x32 => self.d = self.swap(self.d),
            0x33 => self.e = self.swap(self.e),
            0x34 => self.h = self.swap(self.h),
            0x35 => self.l = self.swap(self.l),
            0x36 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.swap(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x37 => self.a = self.swap(self.a),
            0x38 => self.b = self.srl(self.b),  
            0x39 => self.c = self.srl(self.c),
            0x3A => self.d = self.srl(self.d),
            0x3B => self.e = self.srl(self.e),
            0x3C => self.h = self.srl(self.h),
            0x3D => self.l = self.srl(self.l),
            0x3E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                let result = self.srl(temp);
                self.mmu.borrow_mut().write_byte(self.get_HL(), result);
            },
            0x3F => self.a = self.srl(self.a),

            0x40 => self.bit(0, self.b),
            0x41 => self.bit(0, self.c),
            0x42 => self.bit(0, self.d),
            0x43 => self.bit(0, self.e),
            0x44 => self.bit(0, self.h),
            0x45 => self.bit(0, self.l),
            0x46 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(0, temp);
            },
            0x47 => self.bit(0, self.a),
            0x48 => self.bit(1, self.b),
            0x49 => self.bit(1, self.c),
            0x4A => self.bit(1, self.d),
            0x4B => self.bit(1, self.e),
            0x4C => self.bit(1, self.h),
            0x4D => self.bit(1, self.l),
            0x4E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(1, temp);
            },
            0x4F => self.bit(1, self.a),

            0x50 => self.bit(2, self.b),
            0x51 => self.bit(2, self.c),
            0x52 => self.bit(2, self.d),
            0x53 => self.bit(2, self.e),
            0x54 => self.bit(2, self.h),
            0x55 => self.bit(2, self.l),
            0x56 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(2, temp);
            },
            0x57 => self.bit(2, self.a),
            0x58 => self.bit(3, self.b),
            0x59 => self.bit(3, self.c),
            0x5A => self.bit(3, self.d),
            0x5B => self.bit(3, self.e),
            0x5C => self.bit(3, self.h),
            0x5D => self.bit(3, self.l),
            0x5E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(3, temp);
            },
            0x5F => self.bit(3, self.a),

            0x60 => self.bit(4, self.b),
            0x61 => self.bit(4, self.c),
            0x62 => self.bit(4, self.d),
            0x63 => self.bit(4, self.e),
            0x64 => self.bit(4, self.h),
            0x65 => self.bit(4, self.l),
            0x66 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(4, temp);
            },
            0x67 => self.bit(4, self.a),
            0x68 => self.bit(5, self.b),
            0x69 => self.bit(5, self.c),
            0x6A => self.bit(5, self.d),
            0x6B => self.bit(5, self.e),
            0x6C => self.bit(5, self.h),
            0x6D => self.bit(5, self.l),
            0x6E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(5, temp);
            },
            0x6F => self.bit(5, self.a),

            0x70 => self.bit(6, self.b),
            0x71 => self.bit(6, self.c),
            0x72 => self.bit(6, self.d),
            0x73 => self.bit(6, self.e),
            0x74 => self.bit(6, self.h),
            0x75 => self.bit(6, self.l),
            0x76 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(6, temp);
            },
            0x77 => self.bit(6, self.a),
            0x78 => self.bit(7, self.b),
            0x79 => self.bit(7, self.c),
            0x7A => self.bit(7, self.d),
            0x7B => self.bit(7, self.e),
            0x7C => self.bit(7, self.h),
            0x7D => self.bit(7, self.l),
            0x7E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.bit(7, temp);
            },
            0x7F => self.bit(7, self.a),

            0x80 => self.b = self.res(0, self.b),
            0x81 => self.c = self.res(0, self.c),
            0x82 => self.d = self.res(0, self.d),
            0x83 => self.e = self.res(0, self.e),
            0x84 => self.h = self.res(0, self.h),
            0x85 => self.l = self.res(0, self.l),
            0x86 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(0, temp));
            },
            0x87 => self.a = self.res(0, self.a),
            0x88 => self.b = self.res(1, self.b),
            0x89 => self.c = self.res(1, self.c),
            0x8A => self.d = self.res(1, self.d),
            0x8B => self.e = self.res(1, self.e),
            0x8C => self.h = self.res(1, self.h),
            0x8D => self.l = self.res(1, self.l),
            0x8E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(1, temp));
            },
            0x8F => self.a = self.res(1, self.a),

            0x90 => self.b = self.res(2, self.b),
            0x91 => self.c = self.res(2, self.c),
            0x92 => self.d = self.res(2, self.d),
            0x93 => self.e = self.res(2, self.e),
            0x94 => self.h = self.res(2, self.h),
            0x95 => self.l = self.res(2, self.l),
            0x96 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(2, temp));
            },
            0x97 => self.a = self.res(2, self.a),

            0x98 => self.b = self.res(3, self.b),
            0x99 => self.c = self.res(3, self.c),
            0x9A => self.d = self.res(3, self.d),
            0x9B => self.e = self.res(3, self.e),
            0x9C => self.h = self.res(3, self.h),
            0x9D => self.l = self.res(3, self.l),
            0x9E => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(3, temp));
            },
            0x9F => self.a = self.res(3, self.a),

            0xA0 => self.b = self.res(4, self.b),
            0xA1 => self.c = self.res(4, self.c),
            0xA2 => self.d = self.res(4, self.d),
            0xA3 => self.e = self.res(4, self.e),
            0xA4 => self.h = self.res(4, self.h),
            0xA5 => self.l = self.res(4, self.l),
            0xA6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(4, temp));
            },
            0xA7 => self.a = self.res(4, self.a),
            0xA8 => self.b = self.res(5, self.b),
            0xA9 => self.c = self.res(5, self.c),
            0xAA => self.d = self.res(5, self.d),
            0xAB => self.e = self.res(5, self.e),
            0xAC => self.h = self.res(5, self.h),
            0xAD => self.l = self.res(5, self.l),
            0xAE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(5, temp));
            },
            0xAF => self.a = self.res(5, self.a),

            0xB0 => self.b = self.res(6, self.b),
            0xB1 => self.c = self.res(6, self.c),
            0xB2 => self.d = self.res(6, self.d),
            0xB3 => self.e = self.res(6, self.e),
            0xB4 => self.h = self.res(6, self.h),
            0xB5 => self.l = self.res(6, self.l),
            0xB6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(6, temp));
            },
            0xB7 => self.a = self.res(6, self.a),

            0xB8 => self.b = self.res(7, self.b),
            0xB9 => self.c = self.res(7, self.c),
            0xBA => self.d = self.res(7, self.d),
            0xBB => self.e = self.res(7, self.e),
            0xBC => self.h = self.res(7, self.h),
            0xBD => self.l = self.res(7, self.l),
            0xBE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.res(7, temp));
            },
            0xBF => self.a = self.res(7, self.a),

            0xC0 => self.b = self.set(0, self.b),
            0xC1 => self.c = self.set(0, self.c),
            0xC2 => self.d = self.set(0, self.d),
            0xC3 => self.e = self.set(0, self.e),
            0xC4 => self.h = self.set(0, self.h),
            0xC5 => self.l = self.set(0, self.l),
            0xC6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(0, temp));
            },
            0xC7 => self.a = self.set(0, self.a),

            0xC8 => self.b = self.set(1, self.b),
            0xC9 => self.c = self.set(1, self.c),
            0xCA => self.d = self.set(1, self.d),
            0xCB => self.e = self.set(1, self.e),
            0xCC => self.h = self.set(1, self.h),
            0xCD => self.l = self.set(1, self.l),
            0xCE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(1, temp));
            },
            0xCF => self.a = self.set(1, self.a),

            0xD0 => self.b = self.set(2, self.b),
            0xD1 => self.c = self.set(2, self.c),
            0xD2 => self.d = self.set(2, self.d),
            0xD3 => self.e = self.set(2, self.e),
            0xD4 => self.h = self.set(2, self.h),
            0xD5 => self.l = self.set(2, self.l),
            0xD6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(2, temp));
            },
            0xD7 => self.a = self.set(2, self.a),

            0xD8 => self.b = self.set(3, self.b),
            0xD9 => self.c = self.set(3, self.c),
            0xDA => self.d = self.set(3, self.d),
            0xDB => self.e = self.set(3, self.e),
            0xDC => self.h = self.set(3, self.h),
            0xDD => self.l = self.set(3, self.l),
            0xDE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(3, temp));
            },
            0xDF => self.a = self.set(3, self.a),

            0xE0 => self.b = self.set(4, self.b),
            0xE1 => self.c = self.set(4, self.c),
            0xE2 => self.d = self.set(4, self.d),
            0xE3 => self.e = self.set(4, self.e),
            0xE4 => self.h = self.set(4, self.h),
            0xE5 => self.l = self.set(4, self.l),
            0xE6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(4, temp));
            },
            0xE7 => self.a = self.set(4, self.a),

            0xE8 => self.b = self.set(5, self.b),
            0xE9 => self.c = self.set(5, self.c),
            0xEA => self.d = self.set(5, self.d),
            0xEB => self.e = self.set(5, self.e),
            0xEC => self.h = self.set(5, self.h),
            0xED => self.l = self.set(5, self.l),
            0xEE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(5, temp));
            },
            0xEF => self.a = self.set(5, self.a),

            0xF0 => self.b = self.set(6, self.b),
            0xF1 => self.c = self.set(6, self.c),
            0xF2 => self.d = self.set(6, self.d),
            0xF3 => self.e = self.set(6, self.e),
            0xF4 => self.h = self.set(6, self.h),
            0xF5 => self.l = self.set(6, self.l),
            0xF6 => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(6, temp));
            },
            0xF7 => self.a = self.set(6, self.a),

            0xF8 => self.b = self.set(7, self.b),
            0xF9 => self.c = self.set(7, self.c),
            0xFA => self.d = self.set(7, self.d),
            0xFB => self.e = self.set(7, self.e),
            0xFC => self.h = self.set(7, self.h),
            0xFD => self.l = self.set(7, self.l),
            0xFE => {
                let temp = self.mmu.borrow().read_byte(self.get_HL());
                self.mmu.borrow_mut().write_byte(self.get_HL(), self.set(7, temp));
            },
            0xFF => self.a = self.set(7, self.a),
        }
    }
}