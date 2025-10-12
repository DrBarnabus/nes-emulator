use crate::apu::LENGTH_TABLE;

pub struct TriangleChannel {
    pub enabled: bool,
    pub length_counter: u8,
    length_counter_halt: bool,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_counter: 0,
            length_counter_halt: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.length_counter = 0;
        }
    }

    /// $4008
    pub fn write_control(&mut self, value: u8) {
        self.length_counter_halt = value & 0x80 != 0;
    }

    /// $400A
    pub fn write_timer_low(&mut self, _value: u8) {
        // TODO: Timer Low
    }

    /// $400B
    pub fn write_timer_high(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        if self.enabled {
            self.length_counter = LENGTH_TABLE[length_index as usize];
        }
    }

    /// Clock the timer (called at CPU rate)
    pub fn clock_timer(&mut self) {
        // TODO: Implement timer
    }

    /// Clock the linear counter (called at 240 Hz in 4-step, 192 Hz in 5-step)
    pub fn clock_linear_counter(&mut self) {
        // TODO: Implement linear counter
    }

    /// Clock the length counter (called at 120 Hz in 4-step, 96 Hz in 5-step)
    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}

impl Default for TriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}
