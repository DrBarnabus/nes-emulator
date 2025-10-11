mod downsampler;

use crate::audio::downsampler::Downsampler;
use anyhow::{Context, Result, anyhow};
use cpal::traits::StreamTrait;
use cpal::{
    SampleFormat, Stream,
    traits::{DeviceTrait, HostTrait},
};
use ringbuf::producer::Producer;
use ringbuf::traits::{Consumer, Observer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};

pub struct AudioOutput {
    _stream: Stream,
    sample_producer: HeapProd<f32>,
    sample_rate: u32,
    downsampler: Downsampler,
}

impl AudioOutput {
    pub fn new(source_rate: f64, use_high_quality: bool) -> Result<AudioOutput> {
        let host = cpal::default_host();
        let device = host.default_output_device().context("Failed to get default output device")?;

        println!("Audio device: {}", device.name()?);

        let config = device.default_output_config().context("Failed to get default output config")?;
        let sample_rate = config.sample_rate().0;

        println!("Audio sample rate: {} Hz", sample_rate);
        println!("Audio sample format: {}", config.sample_format());
        println!("Audio channels: {}", config.channels());

        println!("Output rate: {} Hz", source_rate);
        println!("Output rate: {} Hz", sample_rate);
        println!("Quality: {}", if use_high_quality { "High (Sinc)" } else { "Fast (Linear)" });

        let buffer_size = (sample_rate as usize / 10).next_power_of_two();
        let ring = HeapRb::<f32>::new(buffer_size);
        let (mut producer, consumer) = ring.split();

        for _ in 0..buffer_size / 2 {
            let _ = producer.try_push(0.0);
        }

        let downsampler = Downsampler::new(source_rate, sample_rate as f64, use_high_quality);

        let stream = match config.sample_format() {
            SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), consumer)?,
            SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), consumer)?,
            SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), consumer)?,
            format => return Err(anyhow!("Unsupported sample format: {:?}", format)),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            sample_producer: producer,
            sample_rate,
            downsampler,
        })
    }

    pub fn push_source_sample(&mut self, sample: f32) {
        let output_samples = self.downsampler.process(sample);

        for output in output_samples {
            let _ = self.sample_producer.try_push(output);
        }
    }

    pub fn buffer_available(&self) -> usize {
        self.sample_producer.vacant_len()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn build_stream<T>(device: &cpal::Device, config: &cpal::StreamConfig, mut consumer: HeapCons<f32>) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let sample = consumer.try_pop().unwrap_or(0.0);

                    for channel_sample in frame.iter_mut() {
                        *channel_sample = T::from_sample(sample);
                    }
                }
            },
            |err| eprintln!("An error occurred on audio stream: {}", err),
            None,
        )?;

        Ok(stream)
    }
}
