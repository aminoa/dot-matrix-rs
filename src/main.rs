mod gb;
use gb::GB;

fn main() {
    let rom_path = "./roms/01-special.gb";

    let mut gb = GB::new(rom_path.to_owned());
    gb.run();
}