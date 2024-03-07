use std::f32::consts::PI;

use crate::ring_buffer::RingBuffer;
use crate::lfo::WavetableLFO;

pub struct Vibrato {
    sample_rate_hz: u32,
    num_channels: usize,

    delay_in_secs: f32,
    oscillator_f0: f32,

    delay_in_samples: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum VibratoParam {
    OscillatorF0,
    DelayInSecs,
}

impl Vibrato {
    pub fn new(sample_rate_hz: u32, num_channels: usize) -> Self {
        Vibrato {
            sample_rate_hz: sample_rate_hz,
            num_channels: num_channels,

            delay_in_secs: f32::default(),
            oscillator_f0: f32::default(),

            delay_in_samples: usize::default(),
        }
    }

    pub fn process(&self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let mut delay_line = vec![RingBuffer::<f32>::new(10); self.num_channels];
        for i in 0..input.len() {
            let mut oscillator = WavetableLFO::new(100, self.oscillator_f0, self.sample_rate_hz);
            let mod_freq = oscillator.next_sample();
            let tap = 1 as f32 + self.delay_in_secs + self.delay_in_samples as f32 * mod_freq;
            for channel in 0..self.num_channels {
                delay_line
                output[i][channel] = input[i][channel];
                // delay_line.push(input[i][channel] as f32);
                // output[i][channel] = delay_line.get_frac(tap);
            }
        }

    }

    pub fn set_param(&mut self, param: VibratoParam, value: f32){
        match param {
            VibratoParam::OscillatorF0 => {self.oscillator_f0 = value; },
            VibratoParam::DelayInSecs => {
                self.delay_in_secs = value;
                self.delay_in_samples = (value * self.sample_rate_hz as f32).round() as usize;
            },
        }
    }

    pub fn get_param(&self, param: VibratoParam) -> f32 {
        match param {
            VibratoParam::OscillatorF0 => self.oscillator_f0,
            VibratoParam::DelayInSecs => self.delay_in_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_params () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as u32;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels);

        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        assert!((vibrato.get_param(VibratoParam::DelayInSecs) - delay_in_secs).abs() <= f32::EPSILON);
        assert!((vibrato.get_param(VibratoParam::OscillatorF0) - f0).abs() <= f32::EPSILON);
        assert_eq!(vibrato.delay_in_samples, 120);
    }

    #[test]
    fn zero_input_zero_output () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as u32;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels);
        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        let input = vec![vec![0 as f32; 5]; 24000];
        let mut output = vec![vec![1.0 as f32; 5]; 24000];

        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        vibrato.process(input_slice.as_slice(), output_slice.as_mut_slice());

        for i in 0..24000 {
            for channel in 0..num_channels {
                assert!(output[i][channel].abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn dc_input_dc_output () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as u32;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels);
        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        let input = vec![vec![0.3 as f32; 5]; 24000];
        let mut output = vec![vec![1.0 as f32; 5]; 24000];

        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        vibrato.process(input_slice.as_slice(), output_slice.as_mut_slice());

        for i in 0..24000 {
            for channel in 0..num_channels {
                assert!((output[i][channel] - 0.3).abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn output_equals_delayed_input () {
        let delay_in_secs = 0.0 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as u32;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels);
        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        let frequency = 220.0; // Hz
        let duration = 2.0; // seconds

        // Generate sine wave
        let num_samples = (duration * sample_rate_hz as f32) as usize;
        let mut input = vec![vec![0.0]; num_samples];

        for i in 0..num_samples {
            let t = i as f32 / sample_rate_hz as f32;
            input[i][0] = (2.0 * PI * frequency * t).sin();
        }
        let mut output = vec![vec![1.0 as f32; 5]; 24000];

        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        vibrato.process(input_slice.as_slice(), output_slice.as_mut_slice());

        for i in 0..24000 {
            for channel in 0..num_channels {
                assert!((output[i][channel] - input[i][channel]).abs() <= f32::EPSILON);
            }
        }
    }
}
