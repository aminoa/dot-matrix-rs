mod app;

#[path = "core/apu.rs"]
mod apu;
#[path = "renderer/audio.rs"]
mod audio;
#[path = "core/cart.rs"]
mod cart;
#[path = "core/consts.rs"]
mod consts;
#[path = "core/cpu.rs"]
mod cpu;
#[path = "core/gb.rs"]
mod gb;
#[path = "core/joypad.rs"]
mod joypad;
#[path = "core/mmu.rs"]
mod mmu;
#[path = "core/ppu.rs"]
mod ppu;

#[path = "renderer/video.rs"]
mod video;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    rom: String,

    #[arg(long)]
    turbo: bool,
}

fn main() {
    let cli = Cli::parse();
    let rom_path = cli.rom;
    let turbo = cli.turbo;

    app::run(rom_path, turbo).expect("eframe failed to launch");
}
