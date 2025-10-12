pub struct Envelope {
    pub start: bool,
    decay_level: u8,
    divider: u8,

    constant_volume: bool,
    constant_volume_value: u8
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            start: false,
            decay_level: 0,
            divider: 0,

            constant_volume: false,
            constant_volume_value: 0,
        }
    }

    pub fn clock(&mut self, length_counter_halt: bool, constant_volume: bool, volume: u8) {
        self.constant_volume = constant_volume;
        self.constant_volume_value = volume;

        if self.start {
            self.start = false;
            self.decay_level = 15;
            self.divider = volume;
        } else if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = volume;

            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if length_counter_halt {
                self.decay_level = 15;
            }
        }
    }

    pub fn get_volume(&self) -> u8 {
        if self.constant_volume {
            self.constant_volume_value
        } else {
            self.decay_level
        }
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new()
    }
}
