#[derive(Copy, Clone, Debug)]
pub struct Colour(pub u8, pub u8, pub u8);

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    const NES_WIDTH: usize = 256;
    const NES_HEIGHT: usize = 240;

    pub fn new() -> Self {
        Self {
            data: vec![0; Self::NES_WIDTH * Self::NES_HEIGHT * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        let base = y * 3 * Self::NES_WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = colour.0;
            self.data[base + 1] = colour.1;
            self.data[base + 2] = colour.2;
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
