extern crate minifb;

use crate::cart::Cart;
use crate::joypad::{Joypad, JoypadButton};
use crate::mmu::MMU;
use crate::ppu::PPU;
use std::cell::RefCell;
use std::rc::Rc;

use crate::consts::{FRAME_RATE, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};

pub struct Renderer {
    pub window: Window,
    pub buffer: Vec<u32>,
}

impl Renderer {
    pub fn new(rom_title: String) -> Renderer {
        // Remove any NUL bytes (unsafe for C strings) by truncating at the first NUL.
        let title = match rom_title.find('\0') {
            Some(idx) => rom_title[..idx].to_string(),
            None => rom_title,
        };

        let mut window = Window::new(
            title.as_str(),
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
            WindowOptions {
                resize: false,
                scale: minifb::Scale::X2,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("Failed to create window: {}", e);
        });

        // Limit to max refresh rate
        window.set_target_fps(FRAME_RATE as usize);

        // Create a buffer to hold the pixel data (RGB format for minifb)
        let buffer = vec![0xFFFFFF; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];

        Renderer { window, buffer }
    }

    pub fn update(&mut self, mmu: &mut MMU, ppu: &mut PPU, joypad: &mut Joypad) {
        // Get the framebuffer from the PPU
        let framebuffer = ppu.framebuffer;

        // Convert the grayscale framebuffer to RGBA format for minifb
        // minifb expects 0xRRGGBB format (32-bit unsigned integers)
        for i in 0..framebuffer.len() {
            let gray_value = framebuffer[i];
            // Convert the grayscale value to RGB (same value for R, G, B)
            self.buffer[i] =
                (gray_value as u32) << 16 | (gray_value as u32) << 8 | (gray_value as u32);
        }

        self.handle_input(joypad);

        // Check if window should close
        if !self.window.is_open() || self.window.is_key_down(Key::Escape) {
            println!("Exiting...");
            std::process::exit(0);
        }

        // Display the framebuffer
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
            .unwrap_or_else(|e| {
                panic!("Failed to update window: {}", e);
            });

        // Savestate

        if (self.window.is_key_down(Key::F1)) {
            mmu.savestate();
        } else if (self.window.is_key_down(Key::F2)) {
            mmu.loadstate();
        }
    }

    fn handle_input(&mut self, joypad: &mut Joypad) {
        self.handle_key(joypad, Key::Up, JoypadButton::Up);
        self.handle_key(joypad, Key::Down, JoypadButton::Down);
        self.handle_key(joypad, Key::Left, JoypadButton::Left);
        self.handle_key(joypad, Key::Right, JoypadButton::Right);

        self.handle_key(joypad, Key::Z, JoypadButton::B);
        self.handle_key(joypad, Key::X, JoypadButton::A);

        self.handle_key(joypad, Key::Enter, JoypadButton::Start);
        self.handle_key(joypad, Key::Space, JoypadButton::Select);
    }

    fn handle_key(&self, joypad: &mut Joypad, key: Key, button: JoypadButton) {
        if self.window.is_key_down(key) {
            joypad.press_button(button);
        } else {
            joypad.release_button(button);
        }
    }
}
