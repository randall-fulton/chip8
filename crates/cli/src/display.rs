use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use emulator::display::Display;

/// Display buffer with optional rendering target
///
/// When no canvas is provided, Display runs in "headless" mode.
pub struct SdlDisplay {
    canvas: Option<Canvas<Window>>,
    pixels: [bool; 64 * 32],
}

impl SdlDisplay {
    const ROWS: u16 = 32;
    const COLS: u16 = 64;

    pub fn new(canvas: Option<Canvas<Window>>) -> Self {
        Self {
            canvas,
            pixels: [false; (Self::COLS * Self::ROWS) as usize],
        }
    }
}

impl Display for SdlDisplay {
    /// Blits a sprite to location (x, y), returning true if any pixels were overwritten
    fn blit_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (idx, row) in sprite.iter().enumerate() {
            let pixel_idx = x as usize + (Self::COLS * (y as u16 + idx as u16)) as usize;
            if pixel_idx + 7 > self.pixels.len() {
                continue;
            }

            let existing = pixels_to_byte(&self.pixels[pixel_idx..pixel_idx + 8]);

            self.pixels[pixel_idx..pixel_idx + 8]
                .clone_from_slice(&byte_to_pixels(row ^ existing));

            let collide = (row & existing) != 0;
            collision = collision || collide;
        }
        collision
    }

    /// Reset display to blank state
    fn clear(&mut self) {
        self.pixels = [false; 64 * 32];
    }

    /// Render current pixel buffer to screen
    fn render(&mut self) {
        if let Some(canvas) = &mut self.canvas {
            canvas.clear();

            let (width, height) = canvas.output_size().unwrap();
            let pixel_width = width / Self::COLS as u32;
            let pixel_height = height / Self::ROWS as u32;

            for y in 0..Self::ROWS {
                for x in 0..Self::COLS {
                    if self.pixels[(y * Self::COLS + x) as usize] {
                        canvas.set_draw_color(Color::WHITE);
                    } else {
                        canvas.set_draw_color(Color::BLACK);
                    }
                    canvas
                        .fill_rect(Rect::new(
                            x as i32 * pixel_width as i32,
                            y as i32 * pixel_height as i32,
                            pixel_width,
                            pixel_height,
                        ))
                        .unwrap();
                }
            }

            canvas.present();
        }
    }
}

fn pixels_to_byte(pixels: &[bool]) -> u8 {
    let mut byte = 0;

    for pixel in pixels {
        byte <<= 1;
        byte += if *pixel { 1 } else { 0 }
    }

    byte
}

fn byte_to_pixels(byte: u8) -> [bool; 8] {
    let mut byte = byte;
    let mut pixels = [false; 8];

    for i in 0..8 {
        pixels[7 - i] = (byte & 0x1) != 0;
        byte >>= 1;
    }

    pixels
}
