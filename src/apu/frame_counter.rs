#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameCounterMode {
    FourStep,
    FiveStep,
}

#[derive(Debug, Default)]
pub struct ClockSignals {
    pub clock_envelopes: bool,
    pub clock_length: bool,
    pub irq: bool,
}

pub struct FrameCounter {
    mode: FrameCounterMode,
    cycle_counter: u32,
    irq_inhibit: bool,
    pub irq_flag: bool,

    write_delay_counter: u8,
    pending_write: Option<u8>,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            mode: FrameCounterMode::FourStep,
            cycle_counter: 0,
            irq_inhibit: false,
            irq_flag: false,
            write_delay_counter: 0,
            pending_write: None,
        }
    }
}

impl FrameCounter {
    pub fn write(&mut self, value: u8) {
        self.pending_write = Some(value);
        self.write_delay_counter = 4;
    }

    pub fn clock(&mut self) -> ClockSignals {
        let mut signals = ClockSignals::default();

        if self.write_delay_counter > 0 {
            self.write_delay_counter -= 1;

            if self.write_delay_counter == 0 {
                if let Some(value) = self.pending_write {
                    let new_mode = if value & 0x80 != 0 { FrameCounterMode::FiveStep } else { FrameCounterMode::FourStep };

                    self.irq_inhibit = value & 0x40 != 0;
                    if self.irq_inhibit {
                        self.irq_flag = false;
                    }

                    self.cycle_counter = 0;
                    self.mode = new_mode;

                    if new_mode == FrameCounterMode::FiveStep {
                        signals.clock_envelopes = true;
                        signals.clock_length = true;
                    }

                    self.pending_write = None;
                }
            }
        }

        self.cycle_counter += 1;

        match self.mode {
            FrameCounterMode::FourStep => self.clock_four_step(&mut signals),
            FrameCounterMode::FiveStep => self.clock_five_step(&mut signals),
        }

        signals.irq = self.irq_flag && !self.irq_inhibit;

        let apu_cycle = self.cycle_counter / 2;
        let get_cycle = self.cycle_counter % 2 == 0;

        let interesting_cycle = match self.mode {
            FrameCounterMode::FourStep => {
                match (apu_cycle, get_cycle) {
                    (3728, false) => true,
                    (7456, false) => true,
                    (11185, false) => true,
                    (14914, true) => true,
                    (14914, false) => true,
                    (0, true) => true,
                    _ => false,
                }
            },
            FrameCounterMode::FiveStep => {
                match (apu_cycle, get_cycle) {
                    (3728, false) => true,
                    (7456, false) => true,
                    (11185, false) => true,
                    (14914, false) => true,
                    (18640, false) => true,
                    (0, true) => true,
                    _ => false,
                }
            }
        };

        if interesting_cycle {
            println!("APU Debug: {:?}, {}, {} ({}), {}, {}, {}",
                     self.mode,
                     self.cycle_counter,
                     apu_cycle,
                     if get_cycle { "get" } else { "put" },
                     signals.clock_envelopes,
                     signals.clock_length,
                     signals.irq);
        }

        signals
    }

    fn clock_four_step(&mut self, signals: &mut ClockSignals) {
        match self.cycle_counter {
            7457 => {
                signals.clock_envelopes = true;
            },
            14913 => {
                signals.clock_envelopes = true;
                signals.clock_length = true;
            },
            22371 => {
                signals.clock_envelopes = true;
            },
            29828 => {
                if !self.irq_inhibit {
                    self.irq_flag = true;
                }
            },
            29829 => {
                signals.clock_envelopes = true;
                signals.clock_length = true;

                if !self.irq_inhibit {
                    self.irq_flag = true;
                }
            },
            29830 => {
                self.cycle_counter = 0;
            }
            _ => {},
        }
    }

    fn clock_five_step(&mut self, signals: &mut ClockSignals) {
        match self.cycle_counter {
            7457 => {
                signals.clock_envelopes = true;
            },
            14913 => {
                signals.clock_envelopes = true;
                signals.clock_length = true;
            },
            22371 => {
                signals.clock_envelopes = true;
            },
            29829 => {},
            37281 => {
                signals.clock_envelopes = true;
                signals.clock_length = true;
            }
            37282 => {
                self.cycle_counter = 0;
            }
            _ => {},
        }
    }
}
