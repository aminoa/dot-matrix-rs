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
    let gb = GB::new(cli.rom);
    let renderer = Renderer::new(gb.cart.title.clone());
    run(gb, renderer);
}

pub fn run(mut gb: GB, mut renderer: Renderer) {
    loop {
        let mut current_cycles: u32 = 0;
        while current_cycles < CYCLES_PER_FRAME {
            let instruction = gb.mmu.read_byte(gb.cpu.pc.clone(), &gb.cart, &gb.joypad);

            let instruction_cycles =
                gb.cpu
                    .execute(instruction, &mut gb.mmu, &mut gb.cart, &mut gb.joypad);
            gb.cpu
                .check_interrupts(&mut gb.mmu, &mut gb.cart, &mut gb.joypad);
            gb.cpu.update_timers(
                instruction_cycles as u32,
                &mut gb.mmu,
                &mut gb.cart,
                &mut gb.joypad,
            );
            gb.ppu.update(
                instruction_cycles as u32,
                &mut gb.mmu,
                &mut gb.cpu,
                &mut gb.cart,
                &mut gb.joypad,
            );

            current_cycles += instruction_cycles as u32;

            if gb.mmu.read_byte(0xFF02, &gb.cart, &gb.joypad) == 0x81 {
                print!("{}", gb.mmu.read_byte(0xFF01, &gb.cart, &gb.joypad) as char);
                gb.mmu.write_byte(0xFF02, 0, &mut gb.cart, &mut gb.joypad);
            }
        }
        // current_cycles -= CYCLES_PER_FRAME;
        renderer.update(&mut gb.mmu, &mut gb.ppu, &mut gb.joypad);
    }
}
