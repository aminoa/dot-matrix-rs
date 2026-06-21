use std::time::Duration;

use eframe;
use egui;

use crate::consts::{CYCLES_PER_FRAME, FRAME_RATE, SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::debugger;
use crate::gb::GB;
use crate::renderer::Renderer;

pub struct App {
    gb: GB,
    renderer: Renderer,
    current_cycles: u32,
    enable_debug: bool,
}

impl App {
    pub fn new(rom_path: String, enable_debug: bool) -> Self {
        App {
            gb: GB::new(rom_path),
            renderer: Renderer::new(),
            current_cycles: 0,
            enable_debug: enable_debug,
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
        while self.current_cycles < CYCLES_PER_FRAME {
            let instruction = self.gb.mmu.read_byte(self.gb.cpu.pc, &self.gb.cart, &self.gb.joypad);

            let instruction_cycles = self.gb.cpu.execute(
                instruction,
                &mut self.gb.mmu,
                &mut self.gb.cart,
                &mut self.gb.joypad,
            );
            self.gb.cpu.check_interrupts(&mut self.gb.mmu, &mut self.gb.cart, &mut self.gb.joypad);
            self.gb.cpu.update_timers(
                instruction_cycles as u32,
                &mut self.gb.mmu,
                &mut self.gb.cart,
                &mut self.gb.joypad,
            );
            self.gb.ppu.update(
                instruction_cycles as u32,
                &mut self.gb.mmu,
                &mut self.gb.cpu,
                &mut self.gb.cart,
                &mut self.gb.joypad,
            );

            self.current_cycles += instruction_cycles as u32;

            if self.gb.mmu.read_byte(0xFF02, &self.gb.cart, &self.gb.joypad) == 0x81 {
                print!("{}", self.gb.mmu.read_byte(0xFF01, &self.gb.cart, &self.gb.joypad) as char);
                self.gb.mmu.write_byte(0xFF02, 0, &mut self.gb.cart, &mut self.gb.joypad);
            }
        }
        self.current_cycles -= CYCLES_PER_FRAME;

        self.renderer.update(ui, &mut self.gb.mmu, &mut self.gb.ppu, &mut self.gb.joypad);

        if self.enable_debug {
            debugger::show(ui.ctx(), &self.gb.cpu);
        }

        ui.ctx().request_repaint_after(Duration::from_secs_f32(1.0 / FRAME_RATE));
    }
}
