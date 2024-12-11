mod gb;
mod cpu;
mod mmu;

use gb::GB;
fn main() {
    // read in a file
    // let rom_path = "./roms/01-special.gb";
    let rom_path = "./roms/tetris.gb";

    let mut gb = GB::new(rom_path.to_owned());
    gb.run();
}