use std::time::{Duration, Instant};

use eframe;
use egui;

use crate::audio::AudioRenderer;
use crate::consts::{
    CYCLES_PER_FRAME, FRAME_INTERVAL, FRAME_RATE, SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::gb::GB;
use crate::video::VideoRenderer;

pub struct App {
    gb: GB,
    rom_path: String,
    video_renderer: VideoRenderer,
    audio_renderer: AudioRenderer,
    next_frame_at: Instant,
    turbo: bool,
}

impl App {
    pub fn new(rom_path: String, turbo: bool) -> Self {
        let (audio_rendererer, producer) = AudioRenderer::new();
        let gb = GB::new(&rom_path, producer, audio_rendererer.sample_rate);

        App {
            gb: gb,
            rom_path: rom_path,
            video_renderer: VideoRenderer::new(),
            audio_renderer: audio_rendererer,
            next_frame_at: Instant::now() + FRAME_INTERVAL,
            turbo: turbo,
        }
    }
}

pub fn run(rom_path: String, turbo: bool) -> eframe::Result<()> {
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
        Box::new(|_| Ok(Box::new(App::new(rom_path, turbo)))),
    )
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let target_rate = if !self.turbo { CYCLES_PER_FRAME } else { CYCLES_PER_FRAME * 20 };

        let now = Instant::now();
        if now >= self.next_frame_at {
            while self.gb.current_cycles < target_rate {
                self.gb.step();
            }
            self.gb.current_cycles -= target_rate;
            self.next_frame_at += FRAME_INTERVAL; // accumulator — no drift
        }

        self.video_renderer.update(ui, &mut self.gb, &self.rom_path);
    }

    fn on_exit(&mut self) {
        if self.gb.cart.battery_support {
            self.gb.mmu.saveram(&self.rom_path, &self.gb.cart);
        }
    }
}
