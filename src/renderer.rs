use crate::cart::Cart;
use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::joypad::{Joypad, JoypadButton};
use crate::mmu::MMU;
use crate::ppu::PPU;
use std::time::{Duration, Instant};

pub struct Renderer {
    texture: Option<egui::TextureHandle>,
    autosave_timer: Instant,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer { texture: None, autosave_timer: Instant::now() + Duration::from_secs(10) }
    }

    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        mmu: &mut MMU,
        ppu: &mut PPU,
        joypad: &mut Joypad,
        cart: &mut Cart,
        rom_path: &String,
    ) {
        let pixels: Vec<egui::Color32> =
            ppu.framebuffer.iter().map(|&pixel| egui::Color32::from_gray(pixel)).collect();
        // map pixel bytes into GPU buffer
        let image = egui::ColorImage::new([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], pixels);

        // need to set NEAREST, else texture is blurry (from bilinear filtering)
        let opts = egui::TextureOptions::NEAREST;

        let tex_id = match &mut self.texture {
            Some(handle) => {
                handle.set(image, opts);
                handle.id()
            }
            None => {
                let handle = ui.ctx().load_texture("screen", image, opts);
                let id = handle.id();
                self.texture = Some(handle);
                id
            }
        };

        ui.centered_and_justified(|ui| {
            ui.add(
                // doesn't store image, but ImageSource that references existing texture
                egui::Image::new((tex_id, egui::vec2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32)))
                    .texture_options(opts)
                    .maintain_aspect_ratio(true)
                    .shrink_to_fit(),
            )
        });

        ui.input(|i| {
            for (key, button) in [
                (egui::Key::ArrowUp, JoypadButton::Up),
                (egui::Key::ArrowDown, JoypadButton::Down),
                (egui::Key::ArrowLeft, JoypadButton::Left),
                (egui::Key::ArrowRight, JoypadButton::Right),
                (egui::Key::Z, JoypadButton::B),
                (egui::Key::X, JoypadButton::A),
                (egui::Key::Enter, JoypadButton::Start),
                (egui::Key::Space, JoypadButton::Select),
            ] {
                if i.key_down(key) {
                    joypad.press_button(button);
                } else {
                    joypad.release_button(button);
                }
            }

            if i.key_pressed(egui::Key::F1) {
                mmu.savestate(rom_path);
            }
            if i.key_pressed(egui::Key::F2) {
                mmu.loadstate(rom_path);
            }

            // Dump save every 10 seconds
            if cart.battery_support {
                let now = Instant::now();
                if now > self.autosave_timer {
                    mmu.saveram(rom_path, cart);
                }
            }
        });

        ui.ctx().request_repaint();
    }
}
