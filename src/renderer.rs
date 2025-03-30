extern crate sdl3; 

use crate::ppu::PPU;
use std::rc::Rc;
use std::cell::RefCell; 

use crate::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl3::libc::grantpt;
use sdl3::render::Canvas;
use sdl3::video::Window;
use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::rect::Point;
use rand::Rng;

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
        canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        canvas.clear();
        canvas.present();

        Renderer {
            canvas: canvas,
            ppu: ppu,
            sdl_context: sdl_context
        }
    }

    pub fn update(&mut self) {

        self.canvas.clear();
        let ppu = self.ppu.borrow().framebuffer;
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let pixel = ppu[x as usize][y as usize];
                self.canvas.set_draw_color(Color::RGB(pixel, pixel, pixel));
                self.canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }

        self.canvas.present();

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