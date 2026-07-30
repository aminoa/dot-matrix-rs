// localStorage persistence for the web frontend. Saves are keyed by the
// cartridge title (`rom_id`) since there are no filesystem paths in the
// browser; blobs are base64-encoded because localStorage is string-only.

use crate::cart::Cart;
use crate::cpu::CPU;
use crate::gb::GB;
use crate::mmu::MMU;
use crate::ppu::PPU;
use base64::Engine;

// same component order as GB::savestate, so the format matches native .st files
fn serialize_state(gb: &GB) -> Vec<u8> {
    let mut bytes = Vec::new();
    bincode::serialize_into(&mut bytes, &gb.cpu).expect("serialize cpu");
    bincode::serialize_into(&mut bytes, &gb.ppu).expect("serialize ppu");
    bincode::serialize_into(&mut bytes, &gb.mmu).expect("serialize mmu");
    bincode::serialize_into(&mut bytes, &gb.cart).expect("serialize cart");
    bytes
}

fn deserialize_state(gb: &mut GB, bytes: &[u8]) -> Result<(), bincode::Error> {
    let mut cursor = std::io::Cursor::new(bytes);

    // deserialize everything before touching gb so a corrupt stored state
    // can't leave the emulator half-loaded
    let cpu: CPU = bincode::deserialize_from(&mut cursor)?;
    let ppu: PPU = bincode::deserialize_from(&mut cursor)?;
    let mmu: MMU = bincode::deserialize_from(&mut cursor)?;
    let mut cart: Cart = bincode::deserialize_from(&mut cursor)?;

    cart.rom = std::mem::take(&mut gb.cart.rom);
    gb.cpu = cpu;
    gb.ppu = ppu;
    gb.mmu = mmu;
    gb.cart = cart;
    Ok(())
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn store(key: String, bytes: &[u8]) {
    let Some(storage) = local_storage() else { return };
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    if let Err(e) = storage.set_item(&key, &encoded) {
        web_sys::console::warn_1(&e);
    }
}

fn fetch(key: String) -> Option<Vec<u8>> {
    let encoded = local_storage()?.get_item(&key).ok()??;
    base64::engine::general_purpose::STANDARD.decode(encoded).ok()
}

pub fn save_state(gb: &GB, rom_id: &str) {
    store(format!("dotmatrix.state.{rom_id}"), &serialize_state(gb));
    crate::web_app::notify("State saved");
}

pub fn load_state(gb: &mut GB, rom_id: &str) {
    match fetch(format!("dotmatrix.state.{rom_id}")) {
        Some(bytes) => match deserialize_state(gb, &bytes) {
            Ok(()) => crate::web_app::notify("State loaded"),
            Err(_) => crate::web_app::notify("State load failed"),
        },
        None => crate::web_app::notify("No saved state"),
    }
}

pub fn save_ram(gb: &mut GB, rom_id: &str) {
    store(format!("dotmatrix.sav.{rom_id}"), &gb.cart.ram);
}

pub fn load_ram(gb: &mut GB, rom_id: &str) {
    if let Some(bytes) = fetch(format!("dotmatrix.sav.{rom_id}")) {
        if bytes.len() == gb.cart.ram.len() {
            gb.cart.ram = bytes;
        }
    }
}
