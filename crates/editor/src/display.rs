use std::fmt;

use eframe::egui;
use emulator::display;
use tokio::sync::{mpsc, oneshot};

pub(crate) enum RenderTargetEditorRequest {
    DumpActiveBuffer(
        oneshot::Sender<RenderTargetEditorResponse>,
        egui::TextureHandle,
    ),
}

impl fmt::Debug for RenderTargetEditorRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderTargetEditorRequest::DumpActiveBuffer(_, _) => write!(f, "DumpActiveBuffer"),
        }
    }
}

pub(crate) enum RenderTargetEditorResponse {
    DumpActiveBuffer(egui::TextureHandle),
}

impl fmt::Debug for RenderTargetEditorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderTargetEditorResponse::DumpActiveBuffer(_) => write!(f, "DumpActiveBuffer"),
        }
    }
}

type Buffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub(crate) struct RenderTarget {
    w: usize,
    h: usize,
    active_buffer: usize,
    buffers: [Buffer; 2],
    rx: mpsc::Receiver<RenderTargetEditorRequest>,
}

impl RenderTarget {
    pub(crate) fn new(
        width: usize,
        height: usize,
        rx: mpsc::Receiver<RenderTargetEditorRequest>,
    ) -> Self {
        let buffer = image::RgbaImage::new(width as u32, height as u32);
        Self {
            w: width,
            h: height,
            active_buffer: 0,
            buffers: [buffer.clone(), buffer],
            rx,
        }
    }

    pub fn get_active_buffer(&self) -> &Buffer {
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

    pub(crate) fn tick(&mut self) {
        let req = match self.rx.try_recv() {
            Ok(v) => v,
            Err(mpsc::error::TryRecvError::Empty) => return,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                eprintln!("RenderTarget receive channel is dead");
                return;
            }
        };
        let RenderTargetEditorRequest::DumpActiveBuffer(tx, mut texture) = req;
        let buf = self.get_active_buffer();
        let pixels = buf.clone().into_flat_samples();
        let img = egui::ImageData::Color(egui::ColorImage::from_rgba_unmultiplied(
            [self.w, self.h],
            pixels.as_slice(),
        ));
        texture.set(img, egui::TextureFilter::Linear);
        tx.send(RenderTargetEditorResponse::DumpActiveBuffer(texture))
            .expect("unable to send texture back to editor");
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
    texture: egui::TextureHandle,
    tx: mpsc::Sender<RenderTargetEditorRequest>,
}

impl GameWindow {
    pub(crate) fn new(
        width: usize,
        height: usize,
        target: egui::TextureHandle,
        tx: mpsc::Sender<RenderTargetEditorRequest>,
    ) -> Self {
        Self {
            w: width,
            h: height,
            texture: target,
            tx,
        }
    }
}

impl eframe::App for GameWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (tx, rx) = oneshot::channel();
            let texture = self.texture.clone();
            self.tx
                .blocking_send(RenderTargetEditorRequest::DumpActiveBuffer(tx, texture))
                .unwrap();

            let RenderTargetEditorResponse::DumpActiveBuffer(texture) = rx.blocking_recv().unwrap();
            self.texture = texture;
            ui.image(&self.texture, self.texture.size_vec2());
        });
    }
}
