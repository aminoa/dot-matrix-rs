mod gb;
mod cpu;
mod mmu;
mod consts;

use gb::GB;
fn main() {
    // TODO: use clap to get command line args
    // let rom_path = "./roms/01-special.gb";
    let rom_path = "./roms/cpu_instrs/01-special.gb";

    let mut gb = GB::new(rom_path.to_owned());
    gb.run();
}