use std::fs;
use serde_json;

#[path="./cpu.rs"]
mod cpu;
use cpu::CPU;

#[path="./mmu.rs"]
mod mmu;
use mmu::MMU;

pub struct GB {
    pub cycles: i64,
    pub cpu: CPU,
    pub mmu: MMU
}

impl GB {
    pub fn new(rom_path: String) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        let cpu = CPU::new();
        let mmu = MMU::new(rom);

        return GB {
            cycles: 0,
            cpu: cpu,
            mmu: mmu,
        }
    }

    pub fn run(&mut self) {
        let opcodes_file = fs::File::open("./data/opcodes.json").expect("Error: Unable to read the file");
        let parsed_opcodes : serde_json::Value = serde_json::from_reader(opcodes_file).expect("Error: Unable to parse the JSON");
        
        loop {
            let instruction = self.mmu.read_byte(self.cpu.pc.clone());
            let opcode = format!("0x{:02X}", &instruction);

            let instruction_metadata = &parsed_opcodes["unprefixed"][&opcode];
            println!("{}", &instruction_metadata["mnemonic"]);

            self.cpu.pc += instruction_metadata["cycles"][0].as_i64().unwrap() as u16;
        }
    }
}