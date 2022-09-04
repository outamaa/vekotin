use crate::color::Color;

pub struct Canvas<'a> {
    pub buffer: &'a mut [u8],
    pub width: u32,
    pub height: u32,
    // color: ? Assume RGB24 for now?
}

impl<'a> Canvas<'a> {
    pub fn draw_point(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return;
        }
        // Ignore alpha, assume RGB24 for now
        let idx = (3 * self.width as i32 * y + 3 * x) as usize;

        self.buffer[idx] = color.r;
        self.buffer[idx + 1] = color.g;
        self.buffer[idx + 2] = color.b;
    }
}
