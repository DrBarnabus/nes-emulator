use std::collections::VecDeque;

pub struct Downsampler {
    _input_rate: f64,
    _output_rate: f64,
    ratio: f64,
    phase: f64,

    filter: SincFilter,
    use_sinc: bool,

    last_sample: f32,
}

impl Downsampler {
    pub fn new(input_rate: f64, output_rate: f64, use_sinc: bool) -> Self {
        let ratio = output_rate / input_rate;

        Self {
            _input_rate: input_rate,
            _output_rate: output_rate,
            ratio,
            phase: 0.0,
            filter: SincFilter::new(32),
            use_sinc,
            last_sample: 0.0,
        }
    }

    pub fn process(&mut self, sample: f32) -> Vec<f32> {
        let mut outputs = vec![];

        if self.use_sinc {
            self.filter.push_sample(sample);
        }

        self.phase += self.ratio;

        while self.phase >= 1.0 {
            let output = if self.use_sinc {
                self.filter.get_interpolated(1.0 - (self.phase - 1.0) as f32)
            } else {
                let t = (self.phase - 1.0) as f32;
                self.last_sample + (sample - self.last_sample) * (1.0 - t)
            };

            outputs.push(output);
            self.phase -= 1.0;
        }

        self.last_sample = sample;
        outputs
    }
}

struct SincFilter {
    taps: usize,
    buffer: VecDeque<f32>,
    coefficients: Vec<f32>, // Pre-computed sinc coefficients
}

impl SincFilter {
    fn new(taps: usize) -> Self {
        let mut filter = Self {
            taps,
            buffer: VecDeque::with_capacity(taps),
            coefficients: vec![],
        };

        for _ in 0..taps {
            filter.buffer.push_back(0.0);
        }

        filter.generate_coefficients(0.0);
        filter
    }

    fn generate_coefficients(&mut self, delay: f32) {
        self.coefficients.clear();

        let half_taps = self.taps as f32 / 2.0;
        let cutoff = 0.45; // Cutoff frequency (normalised)

        for i in 0..self.taps {
            let x = (i as f32 - half_taps + delay) / half_taps;

            let sinc_val = if x.abs() < 1e-6 {
                cutoff
            } else {
                let x_pi = x * std::f32::consts::PI;
                (x_pi * cutoff).sin() / x_pi
            };

            // Blackman window
            let angle = 2.0 * std::f32::consts::PI * i as f32 / (self.taps - 1) as f32;
            let window = 0.42 - 0.5 * angle.cos() + 0.08 * (2.0 * angle).cos();

            self.coefficients.push(sinc_val * window);
        }

        // Normalise
        let sum: f32 = self.coefficients.iter().sum();
        if sum.abs() > 1e-6 {
            for c in self.coefficients.iter_mut() {
                *c /= sum;
            }
        }
    }

    fn push_sample(&mut self, sample: f32) {
        self.buffer.pop_front();
        self.buffer.push_back(sample);
    }

    fn get_interpolated(&mut self, delay: f32) -> f32 {
        self.generate_coefficients(delay);
        self.buffer.iter().zip(self.coefficients.iter()).map(|(s, c)| s * c).sum()
    }
}
