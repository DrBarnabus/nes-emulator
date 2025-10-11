const FOUR_STEP_CYCLES: [u32; 5] = [
    3728,  // Step 0 -> Step 1 (Quarter Frame)
    7456,  // Step 1 -> Step 2 (Half Frame)
    11184, // Step 2 -> Step 3 (Quarter Frame)
    14914, // Step 3 -> Step 4 (Half Frame + IRQ)
    14915, // Step 4 -> Step 0 (IRQ again, then reset)
];

const FIVE_STEP_CYCLES: [u32; 5] = [
    3728,  // Step 0 -> Step 1 (Quarter frame)
    7456,  // Step 1 -> Step 2 (Half Frame)
    11185, // Step 2 -> Step 3 (Quarter Frame)
    14914, // Step 3 -> Step 4 (no IRQ)
    18640, // Step 4 -> Step 0 (Half Frame, then reset)
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameMode {
    FourStep,
    FiveStep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameEvent {
    QuarterFrame,
    HalfFrame,
    SetIrq,
}

pub struct FrameCounter {
    pub mode: FrameMode,
    pub irq_inhibit: bool,

    pub cycle: u32,
    pub interrupt_flag: bool,

    write_delay: Option<u8>,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            mode: FrameMode::FourStep,
            irq_inhibit: false,
            cycle: 0,
            interrupt_flag: false,
            write_delay: None,
        }
    }
}

impl FrameCounter {
    pub fn write(&mut self, value: u8) {
        self.mode = if value & 0x80 != 0 { FrameMode::FiveStep } else { FrameMode::FourStep };
        self.irq_inhibit = value & 0x40 != 0;

        self.write_delay = Some(2);

        if self.irq_inhibit {
            self.interrupt_flag = false;
        }
    }

    pub fn clock(&mut self) -> Option<FrameEvent> {
        if let Some(delay) = self.write_delay {
            if delay > 0 {
                self.write_delay = Some(delay - 1)
            } else {
                self.write_delay = None;
                self.cycle = 0;

                if self.mode == FrameMode::FiveStep {
                    return Some(FrameEvent::HalfFrame);
                }
            }
        }

        self.cycle += 1;

        let (cycles_table, _last_step) = match self.mode {
            FrameMode::FourStep => (&FOUR_STEP_CYCLES, 4),
            FrameMode::FiveStep => (&FIVE_STEP_CYCLES, 5),
        };

        match self.mode {
            FrameMode::FourStep => match self.cycle {
                c if c == cycles_table[0] => Some(FrameEvent::QuarterFrame),
                c if c == cycles_table[1] => Some(FrameEvent::HalfFrame),
                c if c == cycles_table[2] => Some(FrameEvent::QuarterFrame),
                c if c == cycles_table[3] => {
                    if !self.irq_inhibit {
                        self.interrupt_flag = true;
                    }

                    self.cycle = 0;
                    Some(FrameEvent::SetIrq)
                }
                _ => None,
            },
            FrameMode::FiveStep => match self.cycle {
                c if c == cycles_table[0] => Some(FrameEvent::QuarterFrame),
                c if c == cycles_table[1] => Some(FrameEvent::HalfFrame),
                c if c == cycles_table[2] => Some(FrameEvent::QuarterFrame),
                c if c == cycles_table[3] => None,
                c if c == cycles_table[4] => {
                    self.cycle = 0;
                    Some(FrameEvent::HalfFrame)
                }
                _ => None,
            },
        }
    }

    pub fn get_interrupt_flag(&self) -> bool {
        self.interrupt_flag && !self.irq_inhibit
    }
}
