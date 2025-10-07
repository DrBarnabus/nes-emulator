bitflags::bitflags! {
    pub struct PpuCtrlRegister: u8 {
        const NAMETABLE1 = 0b0000_0001;
        const NAMETABLE2 = 0b0000_0010;
        const VRAM_ADD_INCREMENT = 0b0000_0100;
        const SPRITE_PATTERN_ADDR = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDR = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const MASTER_SLAVE_SELECT = 0b0100_0000;
        const GENERATE_NMI = 0b1000_0000;
    }
}

impl PpuCtrlRegister {
    pub fn new() -> Self {
        Self::from_bits_truncate(0)
    }

    pub fn update(&mut self, value: u8) {
        *self = Self::from_bits_truncate(value);
    }

    pub fn nametable_addr(&self) -> u16 {
        match self.bits() & 0x3 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!("Invalid nametable address"),
        }
    }

    pub fn vram_add_increment(&self) -> u8 {
        if !self.contains(Self::VRAM_ADD_INCREMENT) { 1 } else { 32 }
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        if self.contains(Self::SPRITE_PATTERN_ADDR) { 0x1000 } else { 0x0000 }
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if self.contains(Self::BACKGROUND_PATTERN_ADDR) { 0x1000 } else { 0x0000 }
    }

    pub fn sprite_size(&self) -> u8 {
        if self.contains(Self::SPRITE_SIZE) { 16 } else { 8 }
    }
}

impl Default for PpuCtrlRegister {
    fn default() -> Self {
        Self::new()
    }
}
