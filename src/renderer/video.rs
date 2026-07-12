use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::gb::GB;
use crate::joypad::JoypadButton;
use std::time::{Duration, Instant};

pub struct VideoRenderer {
    texture: Option<egui::TextureHandle>,
    autosave_timer: Instant,
}

impl VideoRenderer {
    pub fn new() -> Self {
        VideoRenderer { texture: None, autosave_timer: Instant::now() + Duration::from_secs(10) }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, gb: &mut GB, rom_path: &String) {
        let pixels: Vec<egui::Color32> =
            gb.ppu.framebuffer.iter().map(|&pixel| egui::Color32::from_gray(pixel)).collect();
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

        let mut do_savetate = false;
        let mut do_loadstate = false;

        let autosave_due = gb.cart.battery_support && Instant::now() > self.autosave_timer;

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
                if i.key_pressed(key) {
                    gb.joypad.press_button(button);
                }
                if i.key_released(key) {
                    gb.joypad.release_button(button);
                }
            }

            if i.key_pressed(egui::Key::F1) {
                do_savetate = true;
            }
            if i.key_pressed(egui::Key::F2) {
                do_loadstate = true;
            }
        });

        if do_savetate {
            gb.savestate(rom_path);
        }
        if do_loadstate {
            gb.loadstate(rom_path);
        }

        if autosave_due {
            gb.mmu.saveram(rom_path, &gb.cart);
        }

        ui.ctx().request_repaint();
    }
}
