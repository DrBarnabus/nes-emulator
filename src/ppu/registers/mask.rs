bitflags::bitflags! {
    pub struct PpuMaskRegister: u8 {
        const GREYSCALE = 0b0000_0001;
        const LEFTMOST_8PXL_BACKGROUND = 0b0000_0010;
        const LEFTMOST_8PXL_SPRITE = 0b0000_0100;
        const SHOW_BACKGROUND = 0b0000_1000;
        const SHOW_SPRITES = 0b0001_0000;
        const EMPHASIZE_RED = 0b0010_0000;
        const EMPHASIZE_GREEN = 0b0100_0000;
        const EMPHASIZE_BLUE = 0b1000_0000;
    }
}

pub enum Colour {
    Red,
    Green,
    Blue,
}

impl PpuMaskRegister {
    pub fn new() -> Self {
        Self::from_bits_truncate(0)
    }

    pub fn emphasize(&self) -> Vec<Colour> {
        let mut result = Vec::<Colour>::new();

        if self.contains(Self::EMPHASIZE_RED) {
            result.push(Colour::Red);
        }
        if self.contains(Self::EMPHASIZE_BLUE) {
            result.push(Colour::Blue);
        }
        if self.contains(Self::EMPHASIZE_GREEN) {
            result.push(Colour::Green);
        }

        result
    }

    pub fn update(&mut self, value: u8) {
        *self = Self::from_bits_truncate(value);
    }
}

impl Default for PpuMaskRegister {
    fn default() -> Self {
        Self::new()
    }
}
