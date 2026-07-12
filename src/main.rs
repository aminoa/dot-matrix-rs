mod app;

#[path = "core/apu.rs"]
mod apu;
#[path = "renderer/audio.rs"]
mod audio; // your cpal glue
#[path = "core/cart.rs"]
mod cart;
#[path = "core/consts.rs"]
mod consts;
#[path = "core/cpu.rs"]
mod cpu;
#[path = "core/gb.rs"]
mod gb; // exposes crate::gb::GB
#[path = "core/joypad.rs"]
mod joypad;
#[path = "core/mmu.rs"]
mod mmu;
#[path = "core/ppu.rs"]
mod ppu;

#[path = "renderer/video.rs"]
mod video; // exposes crate::renderer::Renderer

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
