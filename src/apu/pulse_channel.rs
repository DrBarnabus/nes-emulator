use crate::apu::envelope::Envelope;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% negated (75%)
];

pub struct PulseChannel {
    duty_cycle: u8,
    length_counter_halt: bool,
    constant_volume: bool,
    volume: u8,

    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,

    timer_period: u16,
    timer: u16,
    length_counter: u8,

    enabled: bool,
    duty_position: u8,
    envelope: Envelope,
    sweep_unit: SweepUnit,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            duty_cycle: 0,
            length_counter_halt: false,
            constant_volume: false,
            volume: 0,

            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,

            timer: 0,
            timer_period: 0,
            length_counter: 0,

            enabled: false,
            duty_position: 0,
            envelope: Envelope::default(),
            sweep_unit: SweepUnit::default(),
        }
    }

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

        self.sweep_unit.reload();
    }

    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
    }

    pub fn write_timer_high(&mut self, value: u8) {
        let length_index = (value >> 3) & 0x1F;
        self.length_counter = LENGTH_TABLE[length_index as usize];

        self.timer_period = (self.timer_period & 0x00FF) | ((value & 0x07) as u16) << 8;

        self.duty_position = 0;
        self.envelope.start = true;
    }

    pub fn clock_sweep(&mut self, channel_number: u8) {
        let change_amount = self.timer_period >> self.sweep_shift;

        let target_period = if self.sweep_negate {
            if channel_number == 1 {
                self.timer_period.wrapping_sub(change_amount).wrapping_sub(1)
            } else {
                self.timer_period.wrapping_sub(change_amount)
            }
        } else {
            self.timer_period.wrapping_add(change_amount)
        };

        self.sweep_unit.clock(self.sweep_enabled, self.sweep_period, self.timer_period, target_period);

        if let Some(period_update) = self.sweep_unit.get_period_update() {
            self.timer_period = period_update;
        }
    }

    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.duty_position = (self.duty_position + 1) & 0x07;
        } else {
            self.timer -= 1;
        }
    }

    pub fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn clock_envelope(&mut self) {
        self.envelope.clock(self.length_counter_halt, self.constant_volume, self.volume);
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 {
            return 0.0;
        }

        if self.timer_period < 8 || self.sweep_unit.is_muting() {
            return 0.0;
        }

        let duty_cycle = DUTY_CYCLES[self.duty_cycle as usize][self.duty_position as usize];
        if duty_cycle == 0 {
            return 0.0;
        }

        self.envelope.get_volume() as f32
    }
}

impl Default for PulseChannel {
    fn default() -> Self {
        Self::new()
    }
}

struct SweepUnit {
    divider: u8,
    reload: bool,
    muting: bool,
    period_update: Option<u16>,
}

impl SweepUnit {
    fn new() -> Self {
        Self {
            divider: 0,
            reload: false,
            muting: false,
            period_update: None,
        }
    }

    fn clock(&mut self, enabled: bool, period: u8, timer_period: u16, target_period: u16) {
        self.period_update = None;

        self.muting = timer_period < 8 || target_period > 0x7FF;

        if self.divider == 0 && enabled && !self.muting {
            self.period_update = Some(target_period);
        }

        if self.divider == 0 || self.reload {
            self.divider = period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }
    }

    fn get_period_update(&mut self) -> Option<u16> {
        self.period_update.take()
    }

    fn is_muting(&self) -> bool {
        self.muting
    }

    fn reload(&mut self) {
        self.reload = true;
    }
}

impl Default for SweepUnit {
    fn default() -> Self {
        Self::new()
    }
}
