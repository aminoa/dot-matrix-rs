#[cfg(not(target_arch = "wasm32"))]
mod app;

#[path = "core/apu.rs"]
mod apu;
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

#[path = "renderer/audio.rs"]
mod audio;
#[path = "renderer/video.rs"]
mod video;

#[cfg(target_arch = "wasm32")]
#[path = "web/picker.rs"]
mod picker;
#[cfg(target_arch = "wasm32")]
#[path = "web/storage.rs"]
mod storage;
#[cfg(target_arch = "wasm32")]
#[path = "web/app.rs"]
mod web_app;

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    rom: String,

    #[arg(long)]
    turbo: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let cli = Cli::parse();
    let rom_path = cli.rom;
    let turbo = cli.turbo;

    app::run(rom_path, turbo).expect("eframe failed to launch");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web_app::start();
}
