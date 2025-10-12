use crate::apu::LENGTH_TABLE;

pub struct NoiseChannel {
    pub enabled: bool,
    pub length_counter: u8,
    length_counter_halt: bool,
}

impl NoiseChannel {
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

    /// $400C
    pub fn write_control(&mut self, value: u8) {
        self.length_counter_halt = value & 0x20 != 0;
    }

    /// $400E
    pub fn write_timer_low(&mut self, _value: u8) {
        // TODO: Timer Low
    }

    /// $400F
    pub fn write_timer_high(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        if self.enabled {
            self.length_counter = LENGTH_TABLE[length_index as usize];
        }
    }

    /// Clock the timer (called at CPU/2 rate)
    pub fn clock_timer(&mut self) {
        // TODO: Implement timer
    }

    /// Clock the envelope (called at 240 Hz in 4-step, 192 Hz in 5-step)
    pub fn clock_envelope(&mut self) {
        // TODO: Implement envelope
    }

    /// Clock the length counter (called at 120 Hz in 4-step, 96 Hz in 5-step)
    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}
