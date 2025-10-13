pub struct DmcChannel {
    pub enabled: bool,
    pub bytes_remaining: u16,
}

impl DmcChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            bytes_remaining: 0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.bytes_remaining = 0;
        }
    }

    /// Clock the timer (called at CPU rate)
    pub fn clock_timer(&mut self) {
        // TODO: Implement timer
    }
}

impl Default for DmcChannel {
    fn default() -> Self {
        Self::new()
    }
}
