use std::fs;
#[path="./cpu.rs"]
mod cpu;
use cpu::CPU;

pub struct GB {
    pub cycles: i64,
    pub rom: Vec<u8>,
    pub cpu: Box<CPU>
}

impl GB {
    pub fn new(rom_path: String) -> GB {

        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        let cpu = Box::new(CPU::new());

        return GB {
            cycles: 0,
            rom: rom,
            cpu: cpu
        }
        
    }

    pub fn run(&mut self) {

        loop {
            println!("{}", self.cycles);
            self.cycles += 10;
        }
    }
}