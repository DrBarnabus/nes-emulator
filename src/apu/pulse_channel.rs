const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% negated (75%)
];

#[derive(Default)]
pub struct PulseChannel {
    enabled: bool,

    duty_cycle: u8,
    length_counter_halt: bool,
    constant_volume: bool,
    volume: u8,

    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,

    timer_period: u16,
    timer_value: u16,
    length_counter: u8,

    envelope_start: bool,
    envelope_divider: u8,
    envelope_decay: u8,
    sequence_position: u8,
}

impl PulseChannel {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.length_counter = 0;
        }
    }

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }

    pub fn write_control(&mut self, value: u8) {
        self.duty_cycle = (value >> 6) & 0x03;
        self.length_counter_halt = value & 0x20 != 0;
        self.constant_volume = value & 0x10 != 0;
        self.volume = value & 0x0F;
    }

    pub fn write_sweep(&mut self, value: u8) {
        self.sweep_enabled = value & 0x80 != 0;
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_negate = value & 0x08 != 0;
        self.sweep_shift = value & 0x07;
    }

    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
    }

    pub fn write_timer_high(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        self.length_counter = LENGTH_TABLE[length_index as usize];

        self.timer_period = (self.timer_period & 0x00FF) | ((value & 0x07) as u16) << 8;

        self.envelope_start = true;
        self.sequence_position = 0;
    }

    pub fn clock_sweep(&mut self, _channel_number: u8) {
        // TODO: Implement sweep
    }

    pub fn clock_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            self.sequence_position = (self.sequence_position + 1) & 0x07;
        } else {
            self.timer_value -= 1;
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_divider = self.volume;
        } else if self.envelope_decay > 0 {
            self.envelope_decay -= 1;
        } else {
            self.envelope_divider = self.volume;

            if self.envelope_decay > 0 {
                self.envelope_decay -= 1;
            } else if self.length_counter_halt {
                self.envelope_decay = 15;
            }
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 {
            return 0.0;
        }

        let duty_cycle = DUTY_CYCLES[self.duty_cycle as usize][self.sequence_position as usize];
        if duty_cycle == 0 {
            return 0.0;
        }

        self.get_volume() as f32 / 15.0
    }

    fn get_volume(&self) -> u8 {
        if self.constant_volume { self.volume } else { self.envelope_divider }
    }
}
