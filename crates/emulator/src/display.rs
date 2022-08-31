pub trait Display {
    fn blit_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool;
    fn clear(&mut self);
    fn render(&mut self);
}
