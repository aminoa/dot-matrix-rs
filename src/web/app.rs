// The web frontend: eframe web-runner startup and a browser-specific App.
// This is a separate type from the native crate::app::App so the native
// frontend stays exactly as it was before the web port — the browser needs
// a ROM-less start state, a click/drop ROM loader, and localStorage-backed
// persistence, none of which apply natively.

use std::cell::RefCell;
use std::sync::mpsc;

use wasm_bindgen::JsCast;
use web_time::{Duration, Instant};

use crate::audio::AudioRenderer;
use crate::consts::{CYCLES_PER_FRAME, FRAME_INTERVAL};
use crate::gb::GB;
use crate::picker;
use crate::storage;
use crate::video::VideoRenderer;

/// Starts the eframe web runner attached to the page's canvas.
pub fn start() {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("screen")
            .expect("no canvas with id 'screen'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("'screen' element is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    // match the white "screen off" look of the page shell,
                    // regardless of the browser's dark/light preference
                    cc.egui_ctx.set_theme(egui::Theme::Light);
                    Ok(Box::new(WebApp::new()))
                }),
            )
            .await
            .expect("eframe failed to start");
    });
}

thread_local! {
    // ROM bytes in flight from an open file picker (wasm is single-threaded)
    static PICKED_ROM: RefCell<Option<mpsc::Receiver<(String, Vec<u8>)>>> =
        const { RefCell::new(None) };
    // transient status-line notice ("State saved", …) and when it expires
    static NOTICE: RefCell<Option<(String, Instant)>> = const { RefCell::new(None) };
}

/// Shows `msg` on the bezel status line for a couple of seconds, after
/// which it reverts to the playing/paused text. Used by storage.rs to
/// report savestate activity.
pub fn notify(msg: &str) {
    NOTICE.with(|slot| {
        *slot.borrow_mut() = Some((msg.to_string(), Instant::now() + Duration::from_secs(2)));
    });
}

fn take_active_notice() -> Option<String> {
    NOTICE.with(|slot| {
        let mut slot = slot.borrow_mut();
        match &*slot {
            Some((msg, expires)) if Instant::now() < *expires => Some(msg.clone()),
            Some(_) => {
                *slot = None;
                None
            }
            None => None,
        }
    })
}

pub struct WebApp {
    gb: Option<GB>,
    rom_id: String,
    video_renderer: VideoRenderer,
    audio_renderer: Option<AudioRenderer>,
    next_frame_at: Instant,
    paused: bool,
    error: Option<String>,
    last_status: String,
}

impl WebApp {
    fn new() -> Self {
        WebApp {
            gb: None,
            rom_id: String::new(),
            video_renderer: VideoRenderer::new(),
            audio_renderer: None,
            next_frame_at: Instant::now() + FRAME_INTERVAL,
            paused: false,
            error: None,
            last_status: String::new(),
        }
    }

    /// Keeps the bezel status line (a DOM element below the canvas) showing
    /// what's playing, whether it's paused, and any transient notice.
    fn update_status(&mut self) {
        let status = take_active_notice().unwrap_or_else(|| match &self.gb {
            None => "Click the screen to load a ROM".to_string(),
            Some(_) if self.paused => format!("Paused — {}", self.rom_id),
            Some(_) => format!("Playing {}", self.rom_id),
        });

        if status != self.last_status {
            if let Some(el) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("status"))
            {
                el.set_text_content(Some(&status));
            }
            self.last_status = status;
        }
    }

    fn load_rom(&mut self, name: String, rom: Vec<u8>) {
        // Cart::from_rom panics on malformed headers; check the fields it
        // reads so a bad file shows an error instead of killing the app
        if rom.len() < 0x150 || rom[0x148] > 0x07 || rom[0x149] > 0x03 {
            self.error = Some(format!("{name}: not a supported Game Boy ROM"));
            return;
        }

        if let Some(gb) = &mut self.gb {
            if gb.cart.battery_support {
                storage::save_ram(gb, &self.rom_id);
            }
        }

        // the ring-buffer producer moves into GB, so each ROM gets a fresh
        // audio stream; creating it here also ties AudioContext creation to
        // the user gesture that delivered the file, which autoplay policy needs
        let (audio_renderer, producer) = AudioRenderer::new();
        let mut gb = GB::new(rom, producer, audio_renderer.sample_rate);

        // saves are keyed by cartridge title — there are no paths in the browser
        let rom_id = gb.cart.title.clone();
        storage::load_ram(&mut gb, &rom_id);

        self.gb = Some(gb);
        self.rom_id = rom_id;
        self.audio_renderer = Some(audio_renderer);
        self.next_frame_at = Instant::now() + FRAME_INTERVAL;
        self.paused = false;
        self.error = None;
    }

    /// The "screen off" state shown until a ROM is loaded: blank (the
    /// bezel status line carries the load prompt) unless a load failed,
    /// repainting so click polling keeps running.
    fn show_idle_screen(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error {
            ui.centered_and_justified(|ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.label(error);
            });
        }
        ui.ctx().request_repaint();
    }

    /// Loads a ROM dragged onto the screen (the browser delivers the
    /// file's bytes through egui's dropped-files input).
    fn handle_dropped_files(&mut self, ui: &egui::Ui) {
        let dropped = ui.input(|i| i.raw.dropped_files.clone());
        let Some(file) = dropped.into_iter().next() else { return };
        if let Some(bytes) = file.bytes {
            self.load_rom(file.name, bytes.to_vec());
        }
    }

    /// Clicking the screen opens a ROM picker.
    fn handle_screen_click(&mut self, ui: &egui::Ui) {
        // raw pointer input rather than ui.interact: widgets drawn later
        // (the prompt label, the frame image) would occlude an interact
        // region and swallow the click
        let clicked = ui.input(|i| i.pointer.any_click());
        if clicked {
            let (tx, rx) = mpsc::channel();
            PICKED_ROM.with(|slot| *slot.borrow_mut() = Some(rx));
            let ctx = ui.ctx().clone();
            picker::pick_rom_file(move |name, bytes| {
                let _ = tx.send((name, bytes));
                ctx.request_repaint();
            });
        }

        let picked =
            PICKED_ROM.with(|slot| slot.borrow().as_ref().and_then(|rx| rx.try_recv().ok()));
        if let Some((name, bytes)) = picked {
            PICKED_ROM.with(|slot| *slot.borrow_mut() = None);
            self.load_rom(name, bytes);
        }
    }
}

impl eframe::App for WebApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_dropped_files(ui);
        self.handle_screen_click(ui);

        if ui.input(|i| i.key_pressed(egui::Key::P)) {
            self.paused = !self.paused;
        }

        self.update_status();

        let Some(gb) = &mut self.gb else {
            self.show_idle_screen(ui);
            return;
        };

        let now = Instant::now();
        if !self.paused && now >= self.next_frame_at {
            while gb.current_cycles < CYCLES_PER_FRAME {
                gb.step();
            }
            gb.current_cycles -= CYCLES_PER_FRAME;
            self.next_frame_at += FRAME_INTERVAL; // accumulator — no drift
            // if we've fallen far behind (backgrounded tab), resynchronize
            // instead of fast-forwarding to catch up
            if now.duration_since(self.next_frame_at) > Duration::from_millis(250) {
                self.next_frame_at = now + FRAME_INTERVAL;
            }
        } else if self.paused {
            self.next_frame_at = Instant::now() + FRAME_INTERVAL;
        }

        self.video_renderer.update(ui, gb, &self.rom_id);
    }

    // eframe's default clear color is semi-transparent near-black, which
    // composites against the page's white background as murky gray —
    // paint the "screen off" state white instead
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::WHITE.to_normalized_gamma_f32()
    }

    // no on_exit: it never fires reliably in a browser; the autosave timer
    // in the video renderer handles battery saves instead
}
