use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use emulator::display::RenderTarget;

/// Display buffer with optional rendering target
///
/// When no canvas is provided, Display runs in "headless" mode.
pub struct SdlRenderTarget {
    canvas: Option<Canvas<Window>>,
}

impl SdlRenderTarget {
    pub fn new(canvas: Option<Canvas<Window>>) -> Self {
        Self { canvas }
    }
}

impl RenderTarget for SdlRenderTarget {
    fn size(&self) -> (usize, usize) {
        if let Some(canvas) = &self.canvas {
            if let Ok((width, height)) = canvas.output_size() {
                return (width as usize, height as usize);
            }
        }
        (0_usize, 0_usize)
    }

    fn clear(&mut self) {
        if let Some(canvas) = &mut self.canvas {
            canvas.clear()
        }
    }

    fn fill_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: emulator::display::Color,
    ) {
        let canvas = match &mut self.canvas {
            Some(canvas) => canvas,
            None => return,
        };

        canvas.set_draw_color(match color {
            emulator::display::Color::Black => Color::BLACK,
            emulator::display::Color::White => Color::WHITE,
        });
        canvas
            .fill_rect(Rect::new(x as i32, y as i32, w as u32, h as u32))
            .unwrap();
    }

    fn present(&mut self) {
        if let Some(canvas) = &mut self.canvas {
            canvas.present()
        }
    }
}
