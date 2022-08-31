use eframe::egui;
use emulator::display;

pub(crate) struct GameWindow {
    w: usize,
    h: usize,
    buffer: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    texture: egui::TextureHandle,
}

impl GameWindow {
    pub(crate) fn new(width: usize, height: usize, target: egui::TextureHandle) -> Self {
        let buffer = image::RgbImage::new(width as u32, height as u32);
        Self {
            w: width,
            h: height,
            buffer,
            texture: target,
        }
    }
}

impl display::RenderTarget for GameWindow {
    fn present(&mut self) {
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [self.buffer.width() as usize, self.buffer.height() as usize],
            self.buffer.as_flat_samples().as_slice(),
        );
        self.texture
            .set(egui::ImageData::Color(img), egui::TextureFilter::Linear);
    }

    fn clear(&mut self) {
        self.texture.set(
            egui::ImageData::Color(egui::ColorImage::new(
                [self.w, self.h],
                egui::Color32::BLACK,
            )),
            egui::TextureFilter::Linear,
        );
    }

    fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: display::Color) {
        for x in x..=x+w {
            for y in y..=y+h {
                let pixel = match color {
                    display::Color::Black => image::Rgb([0, 0, 0]),
                    display::Color::White => image::Rgb([255, 255, 255]),
                };
                self.buffer.put_pixel(x as u32, y as u32, pixel);
            }
        }
    }
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
