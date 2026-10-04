use std::time::{Duration, Instant};

use eframe::{self, CreationContext};
use egui;

use crate::audio::AudioRenderer;
use crate::consts::{
    CYCLES_PER_FRAME, FRAME_INTERVAL, FRAME_RATE, SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::gb::GB;
use crate::video::VideoRenderer;

const TURBO_BUDGET: Duration = Duration::from_millis(16);

pub struct App {
    gb: GB,
    rom_path: String,
    video_renderer: VideoRenderer,
    audio_renderer: AudioRenderer,
    next_frame_at: Instant,
    turbo: bool,
    paused: bool,

    fps_count: u32,
    fps_window_start: Instant,
}

impl App {
    pub fn new(rom_path: String, turbo: bool) -> Self {
        let (audio_rendererer, producer) =
            AudioRenderer::new().expect("Error: Unable to initialize audio");
        let rom = std::fs::read(&rom_path).expect("Error: Unable to read the file");
        let gb = GB::new(rom, producer, audio_rendererer.sample_rate);

        App {
            gb: gb,
            rom_path: rom_path,
            video_renderer: VideoRenderer::new(),
            audio_renderer: audio_rendererer,
            next_frame_at: Instant::now() + FRAME_INTERVAL,
            turbo: turbo,
            paused: false,
            fps_count: 0,
            fps_window_start: Instant::now(),
        }
    }
}

pub fn run(rom_path: String, turbo: bool) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Dot Matrix").with_inner_size([
            (SCREEN_WIDTH * SCALE_FACTOR) as f32,
            (SCREEN_HEIGHT * SCALE_FACTOR) as f32,
        ]),
        vsync: !turbo,
        ..Default::default()
    };

    let app = App::new(rom_path, turbo);
    eframe::run_native("Dot Matrix", native_options, Box::new(|_| Ok(Box::new(app))))
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(egui::Key::P)) {
            self.paused = !self.paused;
        }

        let now = Instant::now();
        if self.paused {
            self.next_frame_at = now + FRAME_INTERVAL;
        } else if self.turbo {
            // run whole frames until the budget is spent, then let egui paint
            while now.elapsed() < TURBO_BUDGET {
                while self.gb.current_cycles < CYCLES_PER_FRAME {
                    self.gb.step();
                }
                self.gb.current_cycles -= CYCLES_PER_FRAME;
                self.fps_count += 1;
            }
        } else if now >= self.next_frame_at {
            while self.gb.current_cycles < CYCLES_PER_FRAME {
                self.gb.step();
            }
            self.gb.current_cycles -= CYCLES_PER_FRAME;
            self.next_frame_at += FRAME_INTERVAL;
            self.fps_count += 1;
        }

        let elapsed = now.duration_since(self.fps_window_start);
        if elapsed >= Duration::from_secs(1) {
            println!("FPS: {:.1}", self.fps_count as f32 / elapsed.as_secs_f32());
            self.fps_count = 0;
            self.fps_window_start = now;
        }

        self.video_renderer.update(ui, &mut self.gb, &self.rom_path);
    }

    fn on_exit(&mut self) {
        if self.gb.cart.battery_support {
            self.gb.mmu.saveram(&self.rom_path, &self.gb.cart);
        }
    }
}
