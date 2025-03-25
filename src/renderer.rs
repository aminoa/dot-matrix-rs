extern crate sdl3; 

use crate::ppu::PPU;
use std::rc::Rc;
use std::cell::RefCell; // Gives interior mutability to the RC object (allows mutiple accesses) by delaying checks to runtime

use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl3::render::Canvas;
use sdl3::video::Window;
use sdl3::pixels::Color;
use sdl3::event::Event;

pub struct Renderer {
    pub canvas: Canvas<Window>,
    pub ppu: Rc<RefCell<PPU>>,
    pub sdl_context: sdl3::Sdl
}

impl Renderer {
    pub fn new(ppu: Rc<RefCell<PPU>>) -> Renderer {
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        // let event_subsystem = sdl_context.event().unwrap();

        let window = video_subsystem
            .window("Dot Matrix", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Renderer {
            canvas: canvas,
            ppu: ppu,
            sdl_context: sdl_context
        }
    }

    pub fn update(&mut self) {
        for event in self.sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("Exiting...");
                    return;
                }
                _ => {}
            }
        }
    }
}