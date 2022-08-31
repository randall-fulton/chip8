use emulator::display;
use eframe::egui;

pub(crate) struct GameWindow {
    buffer: egui::ColorImage,
    texture: egui::TextureHandle,
}

impl GameWindow {
    pub(crate) fn new(width: usize, height: usize, target: egui::TextureHandle) -> Self {
        Self{
            buffer: egui::ColorImage::new([width, height], egui::Color32::WHITE),
            texture: target,
        }
    }
}

impl display::Display for GameWindow {
    fn blit_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn render(&mut self) {
        self.texture.set(egui::ImageData::Color(self.buffer.clone()), egui::TextureFilter::Linear);
    }
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
