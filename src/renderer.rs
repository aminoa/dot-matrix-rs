extern crate minifb;

use crate::ppu::PPU;
use std::cell::RefCell;
use std::rc::Rc;

use crate::consts::{FRAME_RATE, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};

pub struct Renderer {
    pub window: Window,
    pub buffer: Vec<u32>,
    pub ppu: Rc<RefCell<PPU>>,
}

impl Renderer {
    pub fn new(ppu: Rc<RefCell<PPU>>) -> Renderer {
        let mut window = Window::new(
            "Dot Matrix",
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

        // Update the window with the new pixel data
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
    }
}
