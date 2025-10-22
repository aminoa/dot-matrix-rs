use crate::cart::Cart;
use crate::consts::{CB_OPCODES, CYCLES_PER_FRAME, OPCODES};
use crate::cpu::CPU;
use crate::joypad::Joypad;
use crate::mmu::MMU;
use crate::ppu::PPU;
use crate::renderer::Renderer;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct GB {
    pub cpu: Rc<RefCell<CPU>>,
    pub mmu: Rc<RefCell<MMU>>,
    pub ppu: Rc<RefCell<PPU>>,
    pub renderer: Renderer,
}

impl GB {
    pub fn new(rom_path: String) -> GB {
        // refcell pushes off borrow checking of mutability to runtime, rc allows multiple owners
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        let cart = Rc::new(RefCell::new(Cart::from_rom(rom)));
        let joypad = Rc::new(RefCell::new(Joypad::new()));
        let mmu = Rc::new(RefCell::new(MMU::new(Rc::clone(&cart), Rc::clone(&joypad))));
        let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&mmu))));
        let ppu = Rc::new(RefCell::new(PPU::new(Rc::clone(&mmu), Rc::clone(&cpu))));
        let renderer = Renderer::new(
            Rc::clone(&ppu),
            Rc::clone(&joypad),
            Rc::clone(&cart),
            Rc::clone(&mmu),
        );

        return GB {
            cpu: cpu,
            mmu: mmu,
            ppu: ppu,
            renderer: renderer,
        };
    }

    pub fn run(&mut self) {
        loop {
            let mut current_cycles: u32 = 0;
            while current_cycles < CYCLES_PER_FRAME {
                let instruction = self.mmu.borrow().read_byte(self.cpu.borrow().pc.clone());

                let instruction_cycles = self.cpu.borrow_mut().execute(instruction);
                self.cpu.borrow_mut().check_interrupts();
                self.cpu
                    .borrow_mut()
                    .update_timers(instruction_cycles as u32);
                self.ppu.borrow_mut().update(instruction_cycles as u32);

                current_cycles += instruction_cycles as u32;

                if self.mmu.borrow().read_byte(0xFF02) == 0x81 {
                    print!("{}", self.mmu.borrow().read_byte(0xFF01) as char);
                    self.mmu.borrow_mut().write_byte(0xFF02, 0);
                }
            }

            current_cycles -= CYCLES_PER_FRAME;
            self.renderer.update();
        }
    }
}
