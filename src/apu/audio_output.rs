pub struct Mixer {
    pulse_table: [f32; 31],
    tnd_table: [f32; 203],
}

impl Default for Mixer {
    fn default() -> Self {
        let mut mixer = Self {
            pulse_table: [0.0; 31],
            tnd_table: [0.0; 203],
        };

        // Pre-compute pulse mixing table
        // Formula: 95.88 / ((8128 / (pulse1 + pulse2)) + 100)
        mixer.pulse_table[0] = 0.0;
        for i in 1..31 {
            mixer.pulse_table[i] = 95.88 / ((8128.0 / i as f32) + 100.0)
        }

        // Pre-compute TND mixing table
        // Formula: 159.79 / ((1 / (triangle/8227 + noise/12241 + dmc/22638)) + 100)
        mixer.tnd_table[0] = 0.0;
        for i in 1..203 {
            mixer.tnd_table[i] = 159.79 / ((16367.0 / i as f32) + 100.0);
        }

        mixer
    }
}

impl Mixer {
    pub fn mix(&self, pulse_1: u8, pulse_2: u8, triangle: u8, noise: u8, dmc: u8) -> f32 {
        let pulse_sum = (pulse_1 + pulse_2) as usize;
        let pulse_output = self.pulse_table[pulse_sum.min(30)];

        let tnd_sum = 3 * triangle as usize + 2 * noise as usize + dmc as usize;
        let tnd_output = self.tnd_table[tnd_sum.min(202)];

        let mixed = pulse_output + tnd_output;
        (mixed - 1.0).clamp(-1.0, 1.0)
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
}

pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        let attack_time = 0.003;
        let release_time = 0.100;

        let attack = 1.0 - (-1.0 / (attack_time * sample_rate)).exp();
        let release = 1.0 - (-1.0 / (release_time * sample_rate)).exp();

        Self {
            threshold: 0.7,
            ratio: 4.0,
            attack,
            release,
            envelope: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let input_level = input.abs();

        if input_level > self.envelope {
            self.envelope += (input_level - self.envelope) * self.attack;
        } else {
            self.envelope += (input_level - self.envelope) * self.release;
        }

        let gain = if self.envelope > self.threshold {
            let over = self.envelope - self.threshold;
            let compressed = over / self.ratio;
            (self.threshold + compressed) / self.envelope
        } else {
            1.0
        };

        input * gain
    }
}

pub struct ChannelVolumes {
    pub pulse_1: f32,
    pub pulse_2: f32,
    pub triangle: f32,
    pub noise: f32,
    pub dmc: f32,
}

impl Default for ChannelVolumes {
    fn default() -> Self {
        Self {
            pulse_1: 1.0,
            pulse_2: 1.0,
            triangle: 1.0,
            noise: 1.0,
            dmc: 1.0,
        }
    }
}

pub struct AudioProcessor {
    mixer: Mixer,
    channel_volumes: ChannelVolumes,
    high_pass: HighPassFilter,
    low_pass: LowPassFilter,
    compressor: Compressor,
    master_volume: f32,
}

impl AudioProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            mixer: Mixer::default(),
            channel_volumes: ChannelVolumes::default(),
            high_pass: HighPassFilter::new(90.0, sample_rate),
            low_pass: LowPassFilter::new(14000.0, sample_rate),
            compressor: Compressor::new(sample_rate),
            master_volume: 1.0,
        }
    }

    pub fn process(&mut self, pulse_1: u8, pulse_2: u8, triangle: u8, noise: u8, dmc: u8) -> f32 {
        let pulse_1 = (pulse_1 as f32 * self.channel_volumes.pulse_1) as u8;
        let pulse_2 = (pulse_2 as f32 * self.channel_volumes.pulse_2) as u8;
        let triangle = (triangle as f32 * self.channel_volumes.triangle) as u8;
        let noise = (noise as f32 * self.channel_volumes.noise) as u8;
        let dmc = (dmc as f32 * self.channel_volumes.dmc) as u8;

        let mixed = self.mixer.mix(pulse_1, pulse_2, triangle, noise, dmc);
        let filtered = self.high_pass.process(mixed);
        let filtered = self.low_pass.process(filtered);
        let compressed = self.compressor.process(filtered);
        let output = compressed * self.master_volume;

        self.soft_clip(output)
    }

    fn soft_clip(&self, input: f32) -> f32 {
        if input.abs() > 0.9 {
            input.signum() * (1.0 - (-input.abs() * 2.0).exp())
        } else {
            input
        }
    }
}
