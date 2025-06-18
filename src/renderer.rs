extern crate sdl2; 

use crate::ppu::PPU;
use std::rc::Rc;
use std::cell::{Ref, RefCell}; 

use crate::consts::{FRAME_RATE, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::rect::{Point, Rect};

pub struct Renderer {
    pub canvas: Canvas<Window>,
    pub ppu: Rc<RefCell<PPU>>,
    pub sdl_context: sdl2::Sdl,
}

impl Renderer {
    pub fn new(ppu: Rc<RefCell<PPU>>) -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Dot Matrix", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .always_on_top()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        canvas.clear();
        canvas.present();


        Renderer {
            canvas: canvas,
            ppu: ppu,
            sdl_context: sdl_context,
        }
    }

    pub fn update(&mut self) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();

        self.canvas.clear();

        let framebuffer = self.ppu.borrow().framebuffer.clone();
        texture.update(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT), &framebuffer, SCREEN_WIDTH as usize).unwrap();
        self.canvas.copy(&texture, None, Some(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT))).unwrap();
        self.canvas.present();

        // Handle inputs
        for event in self.sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("Exiting...");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}