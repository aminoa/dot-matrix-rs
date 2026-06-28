use std::time::{Duration, Instant};

use eframe;
use egui;

use crate::consts::{
    CYCLES_PER_FRAME, FRAME_INTERVAL, FRAME_RATE, SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::debugger;
use crate::gb::GB;
use crate::renderer::Renderer;

pub struct App {
    gb: GB,
    rom_path: String,
    renderer: Renderer,
    enable_debug: bool,
    next_frame_at: Instant,
}

impl App {
    pub fn new(rom_path: String, enable_debug: bool) -> Self {
        App {
            gb: GB::new(&rom_path),
            rom_path: rom_path,
            renderer: Renderer::new(),
            enable_debug: enable_debug,
            next_frame_at: Instant::now() + FRAME_INTERVAL,
        }
    }
}

pub fn run(rom_path: String, enable_debug: bool) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Dot Matrix").with_inner_size([
            (SCREEN_WIDTH * SCALE_FACTOR) as f32,
            (SCREEN_HEIGHT * SCALE_FACTOR) as f32,
        ]),
        ..Default::default()
    };

    eframe::run_native(
        "Dot Matrix",
        native_options,
        Box::new(|_| Ok(Box::new(App::new(rom_path, enable_debug)))),
    )
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        if now >= self.next_frame_at {
            while self.gb.current_cycles < CYCLES_PER_FRAME {
                self.gb.step();
            }
            self.gb.current_cycles -= CYCLES_PER_FRAME;
            self.next_frame_at += FRAME_INTERVAL; // accumulator — no drift
        }

        self.renderer.update(
            ui,
            &mut self.gb.mmu,
            &mut self.gb.ppu,
            &mut self.gb.joypad,
            &mut self.gb.cart,
            &self.rom_path,
        );
        if self.enable_debug {
            debugger::show(ui.ctx(), &self.gb.cpu);
        }
    }

    fn on_exit(&mut self) {
        if self.gb.cart.battery_support {
            self.gb.mmu.saveram(&self.rom_path, &self.gb.cart);
        }
    }
}
