use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread::current;
use crate::cpu::CPU;
use crate::mmu::MMU;
use crate::consts::{CB_OPCODES, CYCLES_PER_FRAME, OPCODES};

pub struct GB {
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
            cpu: cpu,
            mmu: mmu,
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut current_cycles: u32 = 0;
            while current_cycles < CYCLES_PER_FRAME {
                let instruction = self.mmu.borrow().read_byte(self.cpu.pc.clone());
                let instruction_cycles = self.cpu.execute(instruction);
                self.cpu.check_interrupts();
                self.cpu.update_timers(instruction_cycles as u32);

                // Blargs 
                if self.mmu.borrow().read_byte(0xFF02) == 0x81 {
                    print!("{}", self.mmu.borrow().read_byte(0xFF01) as char);
                    self.mmu.borrow_mut().write_byte(0xFF02, 0);
                }

                current_cycles += instruction_cycles as u32;
            }

            current_cycles -= CYCLES_PER_FRAME;
            // self.renderer.render(); 
        }
    }
}