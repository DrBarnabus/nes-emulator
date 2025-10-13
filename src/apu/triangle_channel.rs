use crate::apu::LENGTH_TABLE;

const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

pub struct TriangleChannel {
    counter_halt: bool,
    linear_reload: u8,
    linear_counter: u8,
    linear_reload_flag: bool,
    timer_period: u16,
    timer: u16,
    pub length_counter: u8,

    pub enabled: bool,
    sequence_position: u8,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            counter_halt: false,
            linear_reload: 0,
            linear_counter: 0,
            linear_reload_flag: false,
            timer_period: 0,
            timer: 0,
            length_counter: 0,
            enabled: false,
            sequence_position: 0,
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
        self.counter_halt = value & 0x80 != 0;
        self.linear_reload = value & 0x7F;
    }

    /// $400A
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
    }

    /// $400B
    pub fn write_timer_high(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        if self.enabled {
            self.length_counter = LENGTH_TABLE[length_index as usize];
        }

        self.timer_period = (self.timer_period & 0x00FF) | ((value & 0x07) as u16) << 8;

        self.linear_reload_flag = true;
    }

    /// Clock the timer (called at CPU rate)
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;

            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_position = (self.sequence_position + 1) & 0x1F;
            }
        } else {
            self.timer -= 1;
        }
    }

    /// Clock the linear counter (called at 240 Hz in 4-step, 192 Hz in 5-step)
    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload_flag {
            self.linear_counter = self.linear_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.counter_halt {
            self.linear_reload_flag = false;
        }
    }

    /// Clock the length counter (called at 120 Hz in 4-step, 96 Hz in 5-step)
    pub fn clock_length_counter(&mut self) {
        if !self.counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn raw_output(&self) -> u8 {
        if self.timer_period < 2 {
            return 0;
        }

        TRIANGLE_SEQUENCE[self.sequence_position as usize]
    }
}

impl Default for TriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}
