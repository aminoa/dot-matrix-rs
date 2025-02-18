use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

use crate::cpu::CPU;
use crate::mmu::MMU;

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
            println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}", 
                self.cpu.a, self.cpu.f, self.cpu.b, self.cpu.c, self.cpu.d, self.cpu.e, self.cpu.h, self.cpu.l, self.cpu.sp, self.cpu.pc,
                self.mmu.borrow().read_byte(self.cpu.pc),
                self.mmu.borrow().read_byte(self.cpu.pc + 1),
                self.mmu.borrow().read_byte(self.cpu.pc + 2),
                self.mmu.borrow().read_byte(self.cpu.pc + 3));

            let instruction = self.mmu.borrow().read_byte(self.cpu.pc.clone());
            
            self.cpu.execute(instruction);

        }
    }
}