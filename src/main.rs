mod cart;
mod consts;
mod cpu;
mod gb;
mod joypad;
mod mmu;
mod ppu;
mod renderer;

use clap::Parser;
use gb::GB;

use crate::consts::CYCLES_PER_FRAME;
use crate::renderer::Renderer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    rom: String,
}

fn main() {
    let cli = Cli::parse();
    let mut gb = GB::new(cli.rom);
    let renderer = Renderer::new(&gb.ppu, &gb.joypad, &gb.cart, &gb.mmu);
    run(renderer);
}

pub fn run(renderer: Renderer) {
    loop {
        let mut current_cycles: u32 = 0;
        while current_cycles < CYCLES_PER_FRAME {
            let instruction = renderer
                .mmu
                .borrow()
                .read_byte(renderer.cpu.borrow().pc.clone());

            let instruction_cycles = renderer.cpu.borrow_mut().execute(instruction);
            renderer.cpu.borrow_mut().check_interrupts();
            renderer
                .cpu
                .borrow_mut()
                .update_timers(instruction_cycles as u32);
            renderer.ppu.borrow_mut().update(instruction_cycles as u32);

            current_cycles += instruction_cycles as u32;

            if renderer.mmu.borrow().read_byte(0xFF02) == 0x81 {
                print!("{}", renderer.mmu.borrow().read_byte(0xFF01) as char);
                renderer.mmu.borrow_mut().write_byte(0xFF02, 0);
            }
        }

        current_cycles -= CYCLES_PER_FRAME;
        renderer.update();
    }
}
