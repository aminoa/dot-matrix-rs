use crate::apu::APU;
use crate::cart::Cart;
use crate::consts::{CB_OPCODES, CYCLES_PER_FRAME, OPCODES};
use crate::cpu::CPU;
use crate::joypad::Joypad;
use crate::mmu::MMU;
use crate::ppu::PPU;
use ringbuf::HeapProd;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub struct GB {
    pub apu: APU,
    pub cpu: CPU,
    pub mmu: MMU,
    pub ppu: PPU,
    pub cart: Cart,
    pub joypad: Joypad,
    pub current_cycles: u32,
}

impl GB {
    pub fn new(rom_path: &String, sink: HeapProd<f32>) -> GB {
        let rom = fs::read(&rom_path).expect("Error: Unable to read the file");
        return GB {
            apu: APU::new(sink),
            cpu: CPU::new(),
            mmu: MMU::new(),
            ppu: PPU::new(),
            cart: Cart::from_rom(rom),
            joypad: Joypad::new(),
            current_cycles: 0,
        };
    }

    pub fn step(&mut self) {
        let instruction = self.mmu.read_byte(self.cpu.pc, &self.cart, &self.joypad, &mut self.apu);

        let instruction_cycles = self.cpu.execute(
            instruction,
            &mut self.mmu,
            &mut self.cart,
            &mut self.joypad,
            &mut self.apu,
        );
        self.cpu.check_interrupts(&mut self.mmu, &mut self.cart, &mut self.joypad, &mut self.apu);
        self.cpu.update_timers(
            instruction_cycles as u32,
            &mut self.mmu,
            &mut self.cart,
            &mut self.joypad,
            &mut self.apu,
        );
        self.ppu.update(
            instruction_cycles as u32,
            &mut self.mmu,
            &mut self.cpu,
            &mut self.cart,
            &mut self.joypad,
            &mut self.apu,
        );
        self.apu.update(instruction_cycles as u32);

        self.current_cycles += instruction_cycles as u32;
    }

    pub fn savestate(&self, rom_path: &String) {
        let mut path = PathBuf::from(Path::new(rom_path));
        path.set_extension("st");

        let mut bytes = Vec::new();
        bincode::serialize_into(&mut bytes, &self.cpu).expect("serialize cpu");
        bincode::serialize_into(&mut bytes, &self.ppu).expect("serialize ppu");
        bincode::serialize_into(&mut bytes, &self.mmu).expect("serialize mmu");
        bincode::serialize_into(&mut bytes, &self.cart).expect("serialize cart");

        fs::write(&path, &bytes).expect("Failed to write savestate file");
        println!("Savestate saved: {}", path.display());
    }

    pub fn loadstate(&mut self, rom_path: &String) {
        let mut path = PathBuf::from(Path::new(rom_path));
        path.set_extension("st");

        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                println!("Savestate load failed ({}): {}", path.display(), e);
                return;
            }
        };
        let mut cursor = Cursor::new(bytes);

        self.cpu = bincode::deserialize_from(&mut cursor).expect("deserialize cpu");
        self.ppu = bincode::deserialize_from(&mut cursor).expect("deserialize ppu");
        self.mmu = bincode::deserialize_from(&mut cursor).expect("deserialize mmu");

        let rom = std::mem::take(&mut self.cart.rom);
        self.cart = bincode::deserialize_from(&mut cursor).expect("deserialize cart");
        self.cart.rom = rom;

        println!("Savestate loaded: {}", path.display());
    }
}
