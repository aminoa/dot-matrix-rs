mod app;
mod cart;
mod consts;
mod cpu;
mod debugger;
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
    let enable_debug = cli.enable_debug;

    app::run(rom_path, enable_debug).expect("eframe failed to launch");
}
