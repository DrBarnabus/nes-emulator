const DMC_RATE_TABLE: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

pub struct DmcChannel {
    irq_enabled: bool,
    loop_flag: bool,
    rate_index: u8,
    output_level: u8,
    sample_address: u16,
    sample_length: u16,

    timer_period: u16,
    timer: u16,

    pub enabled: bool,
    current_address: u16,
    pub bytes_remaining: u16,

    sample_buffer: u8,
    sample_buffer_empty: bool,

    shift_register: u8,
    bits_remaining: u8,
    silence_flag: bool,

    interrupt_flag: bool,
    needs_init: bool,
}

impl DmcChannel {
    pub fn new() -> Self {
        Self {
            irq_enabled: false,
            loop_flag: false,
            rate_index: 0,
            output_level: 0,
            sample_address: 0,
            sample_length: 0,

            timer_period: DMC_RATE_TABLE[0],
            timer: DMC_RATE_TABLE[0],

            enabled: false,
            current_address: 0,
            bytes_remaining: 0,

            sample_buffer: 0,
            sample_buffer_empty: true,
            shift_register: 0,
            bits_remaining: 0,
            silence_flag: false,
            interrupt_flag: false,
            needs_init: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        let was_enabled = self.enabled;
        self.enabled = enabled;

        if self.enabled {
            if !was_enabled || self.bytes_remaining == 0 {
                self.current_address = self.sample_address;
                self.bytes_remaining = self.sample_length;
                self.needs_init = true;
            }
        } else {
            self.bytes_remaining = 0;
        }
    }

    /// $4010
    pub fn write_flags(&mut self, value: u8) {
        self.irq_enabled = value & 0x80 != 0;
        self.loop_flag = value & 0x40 != 0;
        self.rate_index = value & 0x0F;
        self.timer_period = DMC_RATE_TABLE[self.rate_index as usize];

        if !self.irq_enabled {
            self.interrupt_flag = false;
        }
    }

    /// $4011
    pub fn write_direct_load(&mut self, value: u8) {
        self.output_level = value & 0x7F;
    }

    /// $4012
    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 + (value as u16 * 64);
    }

    /// $4013
    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (value as u16 * 16) + 1;
    }

    /// Clock the timer (called at CPU rate)
    pub fn clock_timer<F>(&mut self, mut read_memory: F)
    where
        F: FnMut(u16) -> u8,
    {
        if self.needs_init && self.sample_buffer_empty && self.bytes_remaining > 0 {
            self.fill_sample_buffer(&mut read_memory);
            self.needs_init = false;
        }

        if self.timer == 0 {
            self.timer = self.timer_period;

            self.clock_output_unit();

            if self.bits_remaining == 0 && self.sample_buffer_empty && self.bytes_remaining > 0 {
                self.fill_sample_buffer(&mut read_memory);
            }
        } else {
            self.timer -= 1;
        }
    }

    fn fill_sample_buffer<F>(&mut self, mut read_memory: F)
    where
        F: FnMut(u16) -> u8,
    {
        self.sample_buffer = read_memory(self.current_address);
        self.sample_buffer_empty = false;

        self.current_address = if self.current_address == 0xFFFF { 0x8000 } else { self.current_address + 1 };

        self.bytes_remaining -= 1;

        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.current_address = self.sample_address;
                self.bytes_remaining = self.sample_length;
            } else if self.irq_enabled {
                self.interrupt_flag = true;
            }
        }
    }

    fn clock_output_unit(&mut self) {
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;

            if self.sample_buffer_empty {
                self.silence_flag = true;
            } else {
                self.silence_flag = false;
                self.shift_register = self.sample_buffer;
                self.sample_buffer_empty = true;
            }
        }

        if !self.silence_flag {
            let bit = self.shift_register & 1;

            if bit == 1 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }

            self.shift_register >>= 1;
        }

        self.bits_remaining -= 1;
    }

    pub fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        (self.output_level as f32 / 63.5) - 1.0
    }

    pub fn get_interrupt(&self) -> bool {
        self.interrupt_flag
    }

    pub fn clear_interrupt(&mut self) {
        self.interrupt_flag = false;
    }
}

impl Default for DmcChannel {
    fn default() -> Self {
        Self::new()
    }
}
