use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::gb::GB;
use crate::joypad::{Joypad, JoypadButton};
use gilrs::{Axis, Button, EventType, Gilrs};
use std::time::{Duration, Instant};

pub struct Renderer {
    texture: Option<egui::TextureHandle>,
    autosave_timer: Instant,
    gilrs: Option<Gilrs>,
    stick_up: bool,
    stick_down: bool,
    stick_left: bool,
    stick_right: bool,
}

const STICK_DEADZONE: f32 = 0.5;

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
            stick_up: false,
            stick_down: false,
            stick_left: false,
            stick_right: false,
        }
    }

    fn update_stick_axis(
        joypad: &mut Joypad,
        value: f32,
        neg_state: &mut bool,
        pos_state: &mut bool,
        neg_button: JoypadButton,
        pos_button: JoypadButton,
    ) {
        let neg = value <= -STICK_DEADZONE;
        let pos = value >= STICK_DEADZONE;
        if neg != *neg_state {
            if neg {
                joypad.press_button(neg_button);
            } else {
                joypad.release_button(neg_button);
            }
            *neg_state = neg;
        }
        if pos != *pos_state {
            if pos {
                joypad.press_button(pos_button);
            } else {
                joypad.release_button(pos_button);
            }
            *pos_state = pos;
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
                    EventType::ButtonPressed(button, _) => match button {
                        Button::RightTrigger => do_savetate = true,
                        Button::LeftTrigger => do_loadstate = true,
                        _ => {
                            if let Some(b) = Self::map_gamepad_button(button) {
                                gb.joypad.press_button(b);
                            }
                        }
                    },
                    EventType::ButtonReleased(button, _) => {
                        if let Some(b) = Self::map_gamepad_button(button) {
                            gb.joypad.release_button(b);
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                        Self::update_stick_axis(
                            &mut gb.joypad,
                            value,
                            &mut self.stick_left,
                            &mut self.stick_right,
                            JoypadButton::Left,
                            JoypadButton::Right,
                        );
                    }
                    EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                        Self::update_stick_axis(
                            &mut gb.joypad,
                            value,
                            &mut self.stick_down,
                            &mut self.stick_up,
                            JoypadButton::Down,
                            JoypadButton::Up,
                        );
                    }
                    _ => {}
                }
            }
        }

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
