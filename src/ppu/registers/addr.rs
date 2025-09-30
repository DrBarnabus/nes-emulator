pub struct PpuAddrRegister {
    value: (u8, u8), // high byte first, low byte second
    latch: bool,
}

impl PpuAddrRegister {
    pub fn new() -> Self {
        Self { value: (0, 0), latch: true }
    }

    pub fn update(&mut self, value: u8) {
        if self.latch {
            self.value.0 = value;
        } else {
            self.value.1 = value;
        }

        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }

        self.latch = !self.latch;
    }

    pub fn increment(&mut self, increment: u8) {
        let low_byte = self.value.1;
        self.value.1 = self.value.1.wrapping_add(increment);

        if low_byte > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        if self.get() > 0x3FFF {
            self.set(self.get() & 0x3FFF);
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | self.value.1 as u16
    }

    fn set(&mut self, value: u16) {
        self.value.0 = (value >> 8) as u8;
        self.value.1 = (value & 0xFF) as u8;
    }
}

impl Default for PpuAddrRegister {
    fn default() -> Self {
        Self::new()
    }
}
