pub enum Color {
    Black,
    White,
}

pub trait RenderTarget {
    fn clear(&mut self);
    fn size(&self) -> (usize, usize);
    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color);
    fn present(&mut self);
}

pub struct Display<T>
where
    T: RenderTarget,
{
    target: T,
    pixels: [bool; 64 * 32],
}

impl<T> Display<T>
where
    T: RenderTarget,
{
    const ROWS: u16 = 32;
    const COLS: u16 = 64;

    pub fn new(target: T) -> Self {
        Self {
            target,
            pixels: [false; (Self::COLS * Self::ROWS) as usize],
        }
    }

    /// Blits a sprite to location (x, y), returning true if any pixels were overwritten
    pub(crate) fn blit_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (idx, row) in sprite.iter().enumerate() {
            let pixel_idx = x as usize + (Self::COLS * (y as u16 + idx as u16)) as usize;
            if pixel_idx + 7 > self.pixels.len() {
                continue;
            }

            let existing = pixels_to_byte(&self.pixels[pixel_idx..pixel_idx + 8]);

            self.pixels[pixel_idx..pixel_idx + 8].clone_from_slice(&byte_to_pixels(row ^ existing));

            let collide = (row & existing) != 0;
            collision = collision || collide;
        }
        collision
    }

    /// Reset display to blank state
    pub(crate) fn clear(&mut self) {
        self.pixels = [false; 64 * 32];
    }

    /// Render current pixel buffer to screen
    pub(crate) fn render(&mut self) {
        self.target.clear();

        let (width, height) = self.target.size();
        let pixel_width = width / Self::COLS as usize;
        let pixel_height = height / Self::ROWS as usize;

        for y in 0..Self::ROWS {
            for x in 0..Self::COLS {
                let draw_color = if self.pixels[(y * Self::COLS + x) as usize] {
                    Color::White
                } else {
                    Color::Black
                };
                self.target.fill_rect(
                    x as usize * pixel_width as usize,
                    y as usize * pixel_height as usize,
                    pixel_width,
                    pixel_height,
                    draw_color,
                );
            }
        }

        self.target.present();
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
