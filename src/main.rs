use std::{io::{self, BufRead}, sync::mpsc::{channel, Receiver}};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample,
};
use cpal::{FromSample, Sample};

// Simple sine oscillator.
pub struct Oscillator {
    pub sample_rate: f32,
    pub frequency: f32,
    pub phase: f32,
}

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

impl Oscillator {
    fn tick(&mut self) -> f32 {
        self.phase += self.frequency * TWO_PI / self.sample_rate;
        self.phase %= TWO_PI;
        self.phase.sin()
    }
}

// This gets called when CPAL sets up our stream.
fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: Receiver<Message>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let mut oscillator = Oscillator {
        sample_rate: config.sample_rate.0 as f32,
        frequency: 440.0,
        phase: 0.0,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // This is the callback in the audio thread!
            // TODO: Receive messages from the main thread to change synthesis parameters.
            // (Hint: `rx.try_recv()`.)
            rx.try_recv().ok().map(|msg| match msg {
                Message::SetFrequency { frequency } => oscillator.frequency = frequency
            });
            process_frame(output, &mut oscillator, num_channels)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

// Messages to control the audio thread.
enum Message {
    SetFrequency { frequency: f32 },
}

fn main() -> anyhow::Result<()> {
    // TODO: Use this channel to communicate with the audio thread!
    let (tx, rx) = channel::<Message>();
    // We give the receiver to our audio callback, and we keep the transmitter for this thread.
    let stream = setup_stream(rx)?;
    // Start playing.
    stream.play()?;
    // Read lines from standard input.
    // Reading from stdin blocks the main thread, but the audio keeps playing because it's being generated in the audio thread.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        // TODO: Parse user input and send messages to the audio thread to control the output.
        // First, try letting the user change the oscillator frequency.
        if line == "q" {
            break;
        }
        let freq = line.parse::<f32>().unwrap();
        tx.send(Message::SetFrequency { frequency: freq }).unwrap();
        // Then come up with some other messages to send!
    }
    Ok(())
}


// CPAL boilerplate.
fn setup_stream(rx: Receiver<Message>) -> Result<cpal::Stream, anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into(), rx),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), rx),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into(), rx),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into(), rx),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into(), rx),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), rx),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into(), rx),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into(), rx),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), rx),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into(), rx),
        sample_format => Err(anyhow::Error::msg(format!(
            "Unsupported sample format '{sample_format}'"
        ))),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

fn process_frame<SampleType>(
    output: &mut [SampleType],
    oscillator: &mut Oscillator,
    num_channels: usize,
) where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = SampleType::from_sample(oscillator.tick());

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
