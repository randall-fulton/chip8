#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{str::FromStr, time::Duration};

use eframe::egui;
use tokio::runtime::Runtime;

mod display;

const WIDTH: usize = 512;
const HEIGHT: usize = 256;

fn main() {
    let rt = Runtime::new().expect("unable to create Runtime");
    let _enter = rt.enter();
    std::thread::spawn(move || {
        // https://github.com/parasyte/egui-tokio-example/blob/main/src/main.rs
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });
    });

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

            let (render_tx, render_rx) =
                tokio::sync::mpsc::channel::<display::RenderTargetEditorRequest>(10);
            let rt = display::RenderTarget::new(WIDTH, HEIGHT, cc.egui_ctx.clone(), render_rx);

            let mut emu = emulator::Chip8::new(emulator::display::Display::new(rt));
            emu.load(std::path::PathBuf::from_str(".\\res\\Maze.ch8").unwrap())
                .unwrap();

            // let egui_ctx = &cc.egui_ctx;
            tokio::spawn(async move {
                loop {
                    emu.tick();
                    emu.display.target.tick(); // NOTE: this hacky af
                }
            });

            let gw = display::GameWindow::new(WIDTH, HEIGHT, target, render_tx);
            Box::new(gw)
        }),
    );
}
