#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Box::new(RenderTarget::new(&cc.egui_ctx))),
    );
}

struct RenderTarget {
    texture: egui::TextureHandle,
}

impl RenderTarget {
    fn new(cc: &egui::Context) -> Self {
        let texture = cc.load_texture("render-target", egui::ColorImage::example(), egui::TextureFilter::Linear);
        Self{
            texture,
        }
    }
}

impl eframe::App for RenderTarget {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
