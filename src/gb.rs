use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use serde_json;

use crate::cpu::CPU;
use crate::mmu::MMU;

pub struct GB {
    pub cycles: i64,
    pub cpu: CPU,
    pub mmu: Rc<RefCell<MMU>>
}

// RC: shared ow

impl GB {
    pub fn new(rom_path: String) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        let mmu = Rc::new(RefCell::new(MMU::new(rom)));
        let cpu = CPU::new(Rc::clone(&mmu));
 
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
            let instruction = self.mmu.borrow().read_byte(self.cpu.pc.clone());
            let opcode = format!("0x{:02X}", &instruction);

            let instruction_metadata = &parsed_opcodes["unprefixed"][&opcode];
            println!("{}", &instruction_metadata["mnemonic"]);

            self.cpu.pc += instruction_metadata["bytes"][0].as_i64().unwrap() as u16;
        }
    }
}