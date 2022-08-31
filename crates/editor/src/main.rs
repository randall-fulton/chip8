#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use emulator::display::Display;

mod display;

fn main() {
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
            let gw = display::GameWindow::new(640, 480, target);
            Box::new(gw)
        }),
    );
}

struct RenderTarget {
    texture: egui::TextureHandle,
}

impl RenderTarget {
    fn new(cc: &egui::Context) -> Self {
        let texture = cc.load_texture(
            "render-target",
            egui::ColorImage::example(),
            egui::TextureFilter::Linear,
        );
        Self { texture }
    }
}

impl eframe::App for RenderTarget {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
