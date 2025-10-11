mod frame_counter;
mod pulse_channel;

use frame_counter::*;
use pulse_channel::*;

pub struct Apu {
    pulse_1: PulseChannel,
    pulse_2: PulseChannel,
    // TODO: triangle: TriangleChannel,
    // TODO: noise: NoiseChannel,
    // TODO: dmc: DmcChannel,
    frame_counter: FrameCounter,

    cycle: u64,

    // Audio processing
    high_pass: HighPassFilter,
    low_pass: LowPassFilter,
}

impl Apu {
    pub fn new() -> Self {
        const NTSC_CPU_FREQUENCY: f32 = 1_789_773.0; // APU runs at CPU clock rate

        Self {
            pulse_1: PulseChannel::default(),
            pulse_2: PulseChannel::default(),
            frame_counter: FrameCounter::default(),
            cycle: 0,

            high_pass: HighPassFilter::new(90.0, NTSC_CPU_FREQUENCY),
            low_pass: LowPassFilter::new(14000.0, NTSC_CPU_FREQUENCY),
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            // Pulse Channel 1
            0x4000..=0x4003 => 0, // Write-only, open bus

            // Pulse Channel 2
            0x4004..=0x4007 => 0, // Write-only, open bus

            // Triangle Channel
            0x4008..=0x400B => 0, // Write-only, open bus

            // Noise Channel
            0x400C..=0x400F => 0, // Write-only, open bus

            // DMC Channel
            0x4010..=0x4013 => 0, // Write-only, open bus

            // Status
            0x4015 => self.read_status(),

            _ => unreachable!(),
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            // Pulse Channel 1
            0x4000 => self.pulse_1.write_control(value),
            0x4001 => self.pulse_1.write_sweep(value),
            0x4002 => self.pulse_1.write_timer_low(value),
            0x4003 => self.pulse_1.write_timer_high(value),

            // Pulse Channel 2
            0x4004 => self.pulse_2.write_control(value),
            0x4005 => self.pulse_2.write_sweep(value),
            0x4006 => self.pulse_2.write_timer_low(value),
            0x4007 => self.pulse_2.write_timer_high(value),

            // Triangle Channel
            0x4008 => {}
            0x4009 => {}
            0x400A => {}
            0x400B => {}

            // Noise Channel
            0x400C => {}
            0x400D => {}
            0x400E => {}
            0x400F => {}

            // DMC Channel
            0x4010 => {}
            0x4011 => {}
            0x4012 => {}
            0x4013 => {}

            // Status
            0x4015 => self.write_status(value),

            // Frame Counter
            0x4017 => self.frame_counter.write(value),

            _ => unreachable!(),
        }
    }

    fn read_status(&mut self) -> u8 {
        let mut status = 0;

        if self.pulse_1.length_counter() > 0 {
            status |= 0x01;
        }
        if self.pulse_2.length_counter() > 0 {
            status |= 0x02;
        }
        // TODO: Triangle Channel
        // TODO: Noise Channel
        // TODO: DMC Channel

        if self.frame_counter.irq_inhibit {
            status |= 0x40;
        }

        // TODO: DMC Interrupt Flag

        self.frame_counter.interrupt_flag = false;

        status
    }

    fn write_status(&mut self, value: u8) {
        self.pulse_1.set_enabled(value & 0x01 != 0);
        self.pulse_2.set_enabled(value & 0x02 != 0);
        // TODO: Triangle Channel Enable
        // TODO: Noise Channel Enable
        // TODO: DMC Channel Enable
    }

    pub fn clock(&mut self) {
        if let Some(event) = self.frame_counter.clock() {
            match event {
                FrameEvent::QuarterFrame => {
                    self.clock_quarter_frame();
                }
                FrameEvent::HalfFrame => {
                    self.clock_half_frame();
                    self.clock_quarter_frame();
                }
                FrameEvent::SetIrq => {}
            }
        }

        self.clock_timers();

        self.cycle += 1;
    }

    pub fn output(&self) -> f32 {
        self.pulse_1.output()
    }

    pub fn filtered_output(&mut self) -> f32 {
        let raw = self.output();

        let high_passed = self.high_pass.process(raw);
        self.low_pass.process(high_passed)
    }

    pub fn get_interrupt_flag(&mut self) -> bool {
        self.frame_counter.get_interrupt_flag()
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse_1.clock_envelope();
        self.pulse_2.clock_envelope();
        // TODO: Noise Channel Clock Envelope

        // TODO: Triangle Channel Clock Linear Counter
    }

    fn clock_half_frame(&mut self) {
        self.pulse_1.clock_length_counter();
        self.pulse_2.clock_length_counter();
        // TODO: Triangle Channel Clock Length Counter
        // TODO: Noise Channel Clock Length Counter

        self.pulse_1.clock_sweep(1);
        self.pulse_1.clock_sweep(2);
    }

    fn clock_timers(&mut self) {
        self.pulse_1.clock_timer();
        self.pulse_2.clock_timer();

        // TODO: Triangle Channel Clock Timer
        // TODO: Noise Channel Clock Timer
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HighPassFilter {
    alpha: f32,
    previous_input: f32,
    previous_output: f32,
}

impl HighPassFilter {
    pub fn new(cutoff_frequency: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_frequency);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);

        Self {
            alpha,
            previous_input: 0.0,
            previous_output: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.alpha * (self.previous_output + input - self.previous_input);
        self.previous_input = input;
        self.previous_output = output;

        output
    }

    pub fn reset(&mut self) {
        self.previous_input = 0.0;
        self.previous_output = 0.0;
    }
}

pub struct LowPassFilter {
    alpha: f32,
    previous_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff_frequency: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_frequency);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        Self { alpha, previous_output: 0.0 }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.alpha * input + (1.0 - self.alpha) * self.previous_output;
        self.previous_output = output;

        output
    }

    pub fn reset(&mut self) {
        self.previous_output = 0.0;
    }
}
