use crate::cart::Cart;
use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::joypad::{Joypad, JoypadButton};
use crate::mmu::MMU;
use crate::ppu::PPU;
use gilrs::{Button, EventType, Gilrs};
use std::time::{Duration, Instant};

pub struct Renderer {
    texture: Option<egui::TextureHandle>,
    autosave_timer: Instant,
    gilrs: Option<Gilrs>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            texture: None,
            autosave_timer: Instant::now() + Duration::from_secs(10),
            gilrs: {
                let g = Gilrs::new().ok();
                if let Some(ref g) = g {
                    for (id, gamepad) in g.gamepads() {
                        println!("gilrs: detected {} ({:?})", gamepad.name(), id);
                    }
                }
                g
            },
        }
    }

    fn map_gamepad_button(button: Button) -> Option<JoypadButton> {
        match button {
            Button::DPadUp => Some(JoypadButton::Up),
            Button::DPadDown => Some(JoypadButton::Down),
            Button::DPadLeft => Some(JoypadButton::Left),
            Button::DPadRight => Some(JoypadButton::Right),
            Button::South => Some(JoypadButton::A),
            Button::West => Some(JoypadButton::B),
            Button::Start => Some(JoypadButton::Start),
            Button::Select => Some(JoypadButton::Select),
            _ => None,
        }
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

        if let Some(gilrs) = &mut self.gilrs {
            while let Some(event) = gilrs.next_event() {
                match event.event {
                    EventType::Connected => {
                        let pad = gilrs.gamepad(event.id);
                        println!("gilrs: connected {} ({:?})", pad.name(), event.id);
                    }
                    EventType::Disconnected => {
                        println!("gilrs: disconnected {:?}", event.id);
                    }
                    EventType::ButtonPressed(button, _) => {
                        if let Some(b) = Self::map_gamepad_button(button) {
                            joypad.press_button(b);
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(b) = Self::map_gamepad_button(button) {
                            joypad.release_button(b);
                        }
                    }
                    _ => {}
                }
            }
        }

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
                    joypad.press_button(button);
                }
                if i.key_released(key) {
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
