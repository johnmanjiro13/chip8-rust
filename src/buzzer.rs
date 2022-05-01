use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};

pub struct Buzzer {
    stream: Stream,
}

impl Buzzer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let mut supported_configs_range = device.supported_output_configs().unwrap();
        let supported_config = supported_configs_range
            .next()
            .unwrap()
            .with_max_sample_rate();
        let sample_format = supported_config.sample_format();
        let config = supported_config.into();

        let stream = match sample_format {
            SampleFormat::F32 => run::<f32>(&device, &config),
            SampleFormat::I16 => run::<i16>(&device, &config),
            SampleFormat::U16 => run::<u16>(&device, &config),
        }
        .unwrap();

        Self { stream }
    }

    pub fn on(&self) {
        self.stream.play().unwrap();
    }

    pub fn off(&self) {
        self.stream.pause().unwrap();
    }
}

fn run<T: Sample>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<Stream, anyhow::Error> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn write_data<T: Sample>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32) {
    for frame in output.chunks_mut(channels) {
        let value: T = Sample::from::<f32>(&next_sample());
        for sample in frame {
            *sample = value;
        }
    }
}
