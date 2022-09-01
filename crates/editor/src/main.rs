#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{str::FromStr, sync::{Arc, Mutex}, time::Duration};

use eframe::egui;

mod display;

const WIDTH: usize = 512;
const HEIGHT: usize = 256;

#[tokio::main]
async fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let target = cc.egui_ctx.load_texture(
                "render-target",
                egui::ColorImage::example(),
                egui::TextureFilter::Linear,
            );

            let rt = display::RenderTarget::new(WIDTH, HEIGHT);
            let emu = Arc::new(Mutex::new(emulator::Chip8::new(
                emulator::display::Display::new(rt),
            )));
            let editor_emu = emu.clone();

            { // TODO: file dialog for this in editor
                let mut lock = emu.lock().unwrap();
                lock.load(std::path::PathBuf::from_str(".\\res\\Sirpinski.ch8").unwrap()).unwrap();
            }

            tokio::spawn(async move {
                loop {
                    let mut emu = emu.lock().unwrap();
                    emu.tick();
                    ::std::thread::sleep(Duration::from_millis(16));
                }
            });

            let gw = display::GameWindow::new(WIDTH, HEIGHT, editor_emu, target);

            Box::new(gw)
        }),
    );
}
