mod audio_output;
mod dmc_channel;
mod envelope;
mod frame_counter;
mod noise_channel;
mod pulse_channel;
mod triangle_channel;

use crate::emulator::NTSC_CPU_FREQUENCY;
use audio_output::*;
use dmc_channel::*;
use frame_counter::*;
use noise_channel::*;
use pulse_channel::*;
use triangle_channel::*;

pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Apu {
    pulse_1: PulseChannel,
    pulse_2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DmcChannel,

    frame_counter: FrameCounter,
    cycle: u64,
    frame_irq: bool,
    dmc_irq: bool,

    audio_processor: AudioProcessor,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            pulse_1: PulseChannel::default(),
            pulse_2: PulseChannel::default(),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
            dmc: DmcChannel::default(),

            frame_counter: FrameCounter::default(),
            cycle: 0,
            frame_irq: false,
            dmc_irq: false,

            audio_processor: AudioProcessor::new(NTSC_CPU_FREQUENCY as f32),
        }
    }
}

impl Apu {
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
            0x4008 => self.triangle.write_control(value),
            0x4009 => { /* Unused */ }
            0x400A => self.triangle.write_timer_low(value),
            0x400B => self.triangle.write_timer_high(value),

            // Noise Channel
            0x400C => self.noise.write_control(value),
            0x400D => { /* Unused */ }
            0x400E => self.noise.write_period(value),
            0x400F => self.noise.write_length_load(value),

            // DMC Channel
            0x4010 => self.dmc.write_flags(value),
            0x4011 => self.dmc.write_direct_load(value),
            0x4012 => self.dmc.write_sample_address(value),
            0x4013 => self.dmc.write_sample_length(value),

            // Status
            0x4015 => self.write_status(value),

            // Frame Counter
            0x4017 => {
                self.frame_counter.write(value);
            }

            _ => unreachable!(),
        }
    }

    fn read_status(&mut self) -> u8 {
        let mut status = 0;

        if self.pulse_1.length_counter > 0 {
            status |= 0x01;
        }

        if self.pulse_2.length_counter > 0 {
            status |= 0x02;
        }

        if self.triangle.length_counter > 0 {
            status |= 0x04;
        }

        if self.noise.length_counter > 0 {
            status |= 0x08;
        }

        if self.dmc.bytes_remaining > 0 {
            status |= 0x10;
        }

        if self.frame_irq {
            status |= 0x40;
        }

        if self.dmc_irq {
            status |= 0x80;
        }

        self.frame_irq = false;
        self.frame_counter.irq_flag = false;

        status
    }

    fn write_status(&mut self, value: u8) {
        self.pulse_1.set_enabled(value & 0x01 != 0);
        self.pulse_2.set_enabled(value & 0x02 != 0);
        self.triangle.set_enabled(value & 0x04 != 0);
        self.noise.set_enabled(value & 0x08 != 0);
        self.dmc.set_enabled(value & 0x10 != 0);

        self.dmc.clear_interrupt()
    }

    pub fn clock<F>(&mut self, read_memory: F)
    where
        F: FnMut(u16) -> u8,
    {
        let signals = self.frame_counter.clock();

        if signals.clock_envelopes {
            self.pulse_1.clock_envelope();
            self.pulse_2.clock_envelope();
            self.noise.clock_envelope();
            self.triangle.clock_linear_counter();
        }

        if signals.clock_length {
            self.pulse_1.clock_length_counter();
            self.pulse_1.clock_sweep(1);
            self.pulse_2.clock_length_counter();
            self.pulse_2.clock_sweep(2);
            self.triangle.clock_length_counter();
            self.noise.clock_length_counter();
        }

        self.frame_irq = signals.irq;

        if (self.cycle & 1) == 0 {
            self.pulse_1.clock_timer();
            self.pulse_2.clock_timer();
            self.noise.clock_timer();
        }

        self.triangle.clock_timer();
        self.dmc.clock_timer(read_memory);

        self.dmc_irq = self.dmc.get_interrupt();

        self.cycle += 1;
    }

    pub fn output(&mut self) -> f32 {
        let pulse_1 = self.pulse_1.raw_output();
        let pulse_2 = self.pulse_2.raw_output();
        let triangle = self.triangle.raw_output();
        let noise = self.noise.raw_output();
        let dmc = self.dmc.output();

        self.audio_processor.process(pulse_1, pulse_2, triangle, noise, dmc)
    }

    pub fn irq_pending(&mut self) -> bool {
        self.frame_irq || self.dmc_irq
    }
}
