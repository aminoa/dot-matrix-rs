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
                // println!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}", 
                //     self.cpu.a, self.cpu.f, self.cpu.b, self.cpu.c, self.cpu.d, self.cpu.e, self.cpu.h, self.cpu.l, self.cpu.sp, self.cpu.pc,
                //     self.mmu.borrow().read_byte(self.cpu.pc),
                //     self.mmu.borrow().read_byte(self.cpu.pc + 1),
                //     self.mmu.borrow().read_byte(self.cpu.pc + 2),
                //     self.mmu.borrow().read_byte(self.cpu.pc + 3));

                let instruction = self.mmu.borrow().read_byte(self.cpu.pc.clone());

                self.cpu.execute(instruction);

                // TODO: handle timing variability for some instructions
                let instruction_cycles = if instruction == 0xCB {
                    CB_OPCODES[instruction as usize].cycles[0] as u32
                } else {
                    OPCODES[instruction as usize].cycles[0] as u32
                };

                self.cpu.check_interrupts();
                self.cpu.update_timers(instruction_cycles);

                // Blargs 
                // if self.mmu.borrow().read_byte(0xFF02) == 0x81 {
                //     print!("{}", self.mmu.borrow().read_byte(0xFF01) as char);
                //     self.mmu.borrow_mut().write_byte(0xFF02, 0);
                // }

                current_cycles += instruction_cycles;
            }

            // self.renderer.render();
        }

    }
}