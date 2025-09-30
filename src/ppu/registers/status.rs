bitflags::bitflags! {
    pub struct PpuStatusRegister: u8 {
        const UNUSED = 0b00000001;
        const UNUSED2 = 0b00000010;
        const UNUSED3 = 0b00000100;
        const UNUSED4 = 0b00001000;
        const UNUSED5 = 0b00010000;
        const SPRITE_OVERFLOW = 0b00100000;
        const SPRITE_0_HIT = 0b01000000;
        const VBLANK_STARTED = 0b10000000;
    }
}

impl PpuStatusRegister {
    pub fn new() -> Self {
        Self::from_bits_truncate(0)
    }
}

impl Default for PpuStatusRegister {
    fn default() -> Self {
        Self::new()
    }
}
