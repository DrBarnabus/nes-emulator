pub struct PpuScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub latch: bool,
}

impl PpuScrollRegister {
    pub fn new() -> Self {
        Self {
            scroll_x: 0,
            scroll_y: 0,
            latch: true,
        }
    }

    pub fn update(&mut self, value: u8) {
        if self.latch {
            self.scroll_x = value;
        } else {
            self.scroll_y = value;
        }

        self.latch = !self.latch;
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }
}

impl Default for PpuScrollRegister {
    fn default() -> Self {
        Self::new()
    }   
}
