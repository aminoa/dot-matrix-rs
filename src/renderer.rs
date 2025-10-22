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
    pub ppu: Rc<RefCell<PPU>>,
    pub joypad: Rc<RefCell<Joypad>>,
    pub cart: Rc<RefCell<Cart>>,
    pub mmu: Rc<RefCell<MMU>>,
}

impl Renderer {
    pub fn new(
        ppu: Rc<RefCell<PPU>>,
        joypad: Rc<RefCell<Joypad>>,
        cart: Rc<RefCell<Cart>>,
        mmu: Rc<RefCell<MMU>>,
    ) -> Renderer {
        let raw_title = cart.borrow().title.clone();
        // Remove any NUL bytes (unsafe for C strings) by truncating at the first NUL.
        let title = match raw_title.find('\0') {
            Some(idx) => raw_title[..idx].to_string(),
            None => raw_title,
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

        Renderer {
            window,
            buffer,
            ppu,
            joypad,
            cart,
            mmu,
        }
    }

    pub fn update(&mut self) {
        // Get the framebuffer from the PPU
        let framebuffer = self.ppu.borrow().framebuffer;

        // Convert the grayscale framebuffer to RGBA format for minifb
        // minifb expects 0xRRGGBB format (32-bit unsigned integers)
        for i in 0..framebuffer.len() {
            let gray_value = framebuffer[i];
            // Convert the grayscale value to RGB (same value for R, G, B)
            self.buffer[i] =
                (gray_value as u32) << 16 | (gray_value as u32) << 8 | (gray_value as u32);
        }

        self.handle_input();

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
            self.mmu.borrow().savestate();
        } else if (self.window.is_key_down(Key::F2)) {
            self.mmu.borrow_mut().loadstate();
        }
    }

    fn handle_input(&mut self) {
        self.handle_key(Key::Up, JoypadButton::Up);
        self.handle_key(Key::Down, JoypadButton::Down);
        self.handle_key(Key::Left, JoypadButton::Left);
        self.handle_key(Key::Right, JoypadButton::Right);

        self.handle_key(Key::Z, JoypadButton::B);
        self.handle_key(Key::X, JoypadButton::A);

        self.handle_key(Key::Enter, JoypadButton::Start);
        self.handle_key(Key::Space, JoypadButton::Select);
    }

    fn handle_key(&self, key: Key, button: JoypadButton) {
        let mut joypad = self.joypad.borrow_mut();
        if self.window.is_key_down(key) {
            joypad.press_button(button);
        } else {
            joypad.release_button(button);
        }
    }
}
