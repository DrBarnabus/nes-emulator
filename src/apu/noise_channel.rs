use crate::apu::LENGTH_TABLE;
use crate::apu::envelope::Envelope;

const NOISE_PERIOD_TABLE: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

pub struct NoiseChannel {
    length_counter_halt: bool,
    constant_volume: bool,
    volume: u8,
    mode: bool, // false = Normal, true = Short Period
    period_index: u8,

    timer_period: u16,
    timer: u16,
    pub length_counter: u8,

    pub enabled: bool,
    shift_register: u16,
    envelope: Envelope,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            length_counter_halt: false,
            constant_volume: false,
            volume: 0,
            mode: false,
            period_index: 0,
            timer_period: 0,
            timer: 0,
            shift_register: 1,
            enabled: false,
            length_counter: 0,
            envelope: Envelope::default(),
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
        self.constant_volume = value & 0x10 != 0;
        self.volume = value & 0x0F;
    }

    /// $400E
    pub fn write_period(&mut self, value: u8) {
        self.mode = value & 0x80 != 0;
        self.period_index = value & 0x0F;
        self.timer_period = NOISE_PERIOD_TABLE[self.period_index as usize];
    }

    /// $400F
    pub fn write_length_load(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        if self.enabled {
            self.length_counter = LENGTH_TABLE[length_index as usize];
        }

        self.envelope.start = true;
    }

    /// Clock the timer (called at CPU/2 rate)
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;

            self.clock_lfsr();
        } else {
            self.timer -= 1;
        }
    }

    fn clock_lfsr(&mut self) {
        let feedback_bit = if self.mode { 6 } else { 1 };
        let feedback = (self.shift_register & 1) ^ ((self.shift_register >> feedback_bit) & 1);

        self.shift_register >>= 1;
        self.shift_register |= feedback << 14;
    }

    /// Clock the envelope (called at 240 Hz in 4-step, 192 Hz in 5-step)
    pub fn clock_envelope(&mut self) {
        self.envelope.clock(self.length_counter_halt, self.constant_volume, self.volume);
    }

    /// Clock the length counter (called at 120 Hz in 4-step, 96 Hz in 5-step)
    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 {
            return 0.0;
        }

        if (self.shift_register & 1) == 1 {
            return 0.0;
        }

        self.envelope.get_volume() as f32 / 15.0
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}
