use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

use crate::cpu::CPU;
use crate::mmu::MMU;
use crate::consts::{OPCODES, CB_OPCODES};

pub struct GB {
    pub cycles: i64,
    pub cpu: CPU,
    pub mmu: Rc<RefCell<MMU>>
}

impl GB {
    pub fn new(rom_path: String) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        // refcell pushes off borrow checking of mutability to runtime, rc allows multiple owners
        let mmu = Rc::new(RefCell::new(MMU::new(rom)));
        let cpu = CPU::new(Rc::clone(&mmu));
 
        return GB {
            cycles: 0,
            cpu: cpu,
            mmu: mmu,
        }
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.mmu.borrow().read_byte(self.cpu.pc.clone());
            
            if instruction == 0xCB {
                let cb_instruction = self.mmu.borrow().read_byte(self.cpu.pc + 1);
                let mnemonic = CB_OPCODES[cb_instruction as usize].mnemonic;
                // println!("PC: {:04X} OP: {:02X} {}", self.cpu.pc, cb_instruction, mnemonic);
            } else {
                let mnemonic = OPCODES[instruction as usize].mnemonic;
                // println!("PC: {:04X} OP: {:02X} {}", self.cpu.pc, instruction, mnemonic);
            }
            self.cpu.execute(instruction);
        }
    }
}