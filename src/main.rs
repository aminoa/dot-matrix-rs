mod gb;
mod cpu;
mod mmu;
mod ppu;
mod consts;
mod renderer;

use gb::GB;
use clap::Parser;

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