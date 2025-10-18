mod consts;
mod cpu;
mod gb;
mod joypad;
mod mmu;
mod ppu;
mod renderer;

use clap::Parser;
use gb::GB;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    rom: String,
}

fn main() {
    let cli = Cli::parse();
    let mut gb = GB::new(cli.rom);
    gb.run();
}
