use std::sync::{Arc, Mutex};

use eframe::egui;
use emulator::display;

type Buffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub(crate) struct RenderTarget {
    w: usize,
    h: usize,
    active_buffer: usize,
    buffers: [Buffer; 2],
}

impl RenderTarget {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        let buffer = image::RgbaImage::new(width as u32, height as u32);
        Self {
            w: width,
            h: height,
            active_buffer: 0,
            buffers: [buffer.clone(), buffer],
        }
    }

    pub(crate) fn get_active_buffer(&self) -> &Buffer {
        assert!(
            self.active_buffer <= 1,
            "targeting more than two render buffers is invalid"
        );
        &self.buffers[self.active_buffer]
    }

    fn get_back_buffer(&mut self) -> &mut Buffer {
        assert!(
            self.active_buffer <= 1,
            "targeting more than two render buffers is invalid"
        );
        let index = if self.active_buffer == 0 { 1 } else { 0 };
        &mut self.buffers[index]
    }
}

impl display::RenderTarget for RenderTarget {
    fn present(&mut self) {
        self.active_buffer = if self.active_buffer == 0 { 1 } else { 0 };
    }

    fn clear(&mut self) {
        self.fill_rect(0, 0, self.w, self.h, display::Color::White);
    }

    fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: display::Color) {
        for x in x..x + w {
            for y in y..y + h {
                let pixel = match color {
                    display::Color::Black => image::Rgba([0, 0, 0, 255]),
                    display::Color::White => image::Rgba([255, 255, 255, 255]),
                };
                self.get_back_buffer().put_pixel(x as u32, y as u32, pixel);
            }
        }
    }
}

pub(crate) struct GameWindow {
    w: usize,
    h: usize,
    emulator: Arc<Mutex<emulator::Chip8<RenderTarget>>>,
    texture: egui::TextureHandle,
}

impl GameWindow {
    pub(crate) fn new(
        width: usize,
        height: usize,
        emulator: Arc<Mutex<emulator::Chip8<RenderTarget>>>,
        target: egui::TextureHandle,
    ) -> Self {
        Self {
            w: width,
            h: height,
            emulator,
            texture: target,
        }
    }
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let buf: Buffer;
            { // ensure the lock is released ASAP so emulator can keep chugging along
                let emu = self
                    .emulator
                    .lock()
                    .expect("failed to obtain lock in GameWindow");
                buf = emu.display.target.get_active_buffer().clone();
            }
            let pixels = buf.as_flat_samples();
            let img = egui::ColorImage::from_rgba_unmultiplied([self.w, self.h], pixels.as_slice());
            // let img = egui::ColorImage::new([self.w, self.h], egui::Color32::WHITE);
            self.texture.set(img, egui::TextureFilter::Linear);
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
