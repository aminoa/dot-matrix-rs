mod app;
mod apu;
mod cart;
mod consts;
mod cpu;
mod gb;
mod joypad;
mod mmu;
mod ppu;
mod renderer;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    rom: String,

    #[arg(long)]
    enable_debug: bool,
}

fn main() {
    let cli = Cli::parse();
    let rom_path = cli.rom;

    app::run(rom_path).expect("eframe failed to launch");
}
