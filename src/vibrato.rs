use crate::ring_buffer::RingBuffer;
use crate::lfo::WavetableLFO;

pub struct Vibrato {
    sample_rate_hz: usize,

    delay_in_secs: f32,
    oscillator_f0: f32,

    delay_lines: Vec<RingBuffer<f32>>
}

#[derive(Debug, Clone, Copy)]
pub enum VibratoParam {
    OscillatorF0,
    DelayInSecs,
}

impl Vibrato {
    pub fn new(sample_rate_hz: usize, num_channels: usize, max_delay_secs: f32) -> Self {
        let mut delay_lines = Vec::with_capacity(num_channels);
        let delay_line_size = 3 * (max_delay_secs * sample_rate_hz as f32).ceil() as usize + 2;
        for _ in 0..num_channels {
            let delay_line = RingBuffer::new(delay_line_size);
            delay_lines.push(delay_line);
        };
        Vibrato {
            sample_rate_hz: sample_rate_hz,

            delay_in_secs: f32::default(),
            oscillator_f0: f32::default(),

            delay_lines: delay_lines,
        }
    }

    pub fn reset(&mut self) {
        for delay_line in &mut self.delay_lines {
            delay_line.reset()
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for i in 0..self.delay_lines.len() {
            let delay_line = &mut self.delay_lines[i];
            let mut oscillator = WavetableLFO::new(100, self.oscillator_f0, self.sample_rate_hz);
            for (x, y) in input[i].iter().zip(output[i].iter_mut()) {
                let mod_freq = oscillator.next_sample();
                let tap = 1 as f32 + self.delay_in_secs + self.delay_in_secs * mod_freq;

                delay_line.push(*x);
                *y = x + delay_line.get_frac(tap);   
            }
        }
    }

    pub fn set_param(&mut self, param: VibratoParam, value: f32){
        match param {
            VibratoParam::OscillatorF0 => {self.oscillator_f0 = value; },
            VibratoParam::DelayInSecs => {
                self.delay_in_secs = value;
                let read_index = self.delay_lines[0].capacity() + self.delay_lines[0].get_write_index() - value as usize;
                for delay_line in self.delay_lines.iter_mut() {
                    delay_line.set_read_index(read_index);
                }
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
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_set_and_get_params () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as usize;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels, delay_in_secs);

        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        assert!((vibrato.get_param(VibratoParam::DelayInSecs) - delay_in_secs).abs() <= f32::EPSILON);
        assert!((vibrato.get_param(VibratoParam::OscillatorF0) - f0).abs() <= f32::EPSILON);
    }

    #[test]
    fn zero_input_zero_output () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as usize;
        let num_channels = 1 as usize;
        let block_size = 1024;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels, delay_in_secs);
        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        let block = vec![vec![0.0_f32; block_size]; num_channels];
        let mut output_block = vec![vec![0.0_f32; block_size]; num_channels];

        let ins = block.iter().map(|c| c.as_slice()).collect::<Vec<&[f32]>>();
        let mut outs = output_block.iter_mut().map(|c| c.as_mut_slice()).collect::<Vec<&mut [f32]>>();
        vibrato.process(ins.as_slice(), outs.as_mut_slice());

        for i in 0..block_size {
            for channel in 0..num_channels {
                assert!(output_block[channel][i].abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn dc_input_dc_output () {
        let delay_in_secs = 0.005 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as usize;
        let num_channels = 1 as usize;
        let block_size = 1024;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels, delay_in_secs);
        vibrato.set_param(VibratoParam::DelayInSecs, delay_in_secs);
        vibrato.set_param(VibratoParam::OscillatorF0, f0);

        let block = vec![vec![0.0_f32; block_size]; num_channels];
        let mut output_block = vec![vec![0.0_f32; block_size]; num_channels];

        let ins = block.iter().map(|c| c.as_slice()).collect::<Vec<&[f32]>>();
        let mut outs = output_block.iter_mut().map(|c| c.as_mut_slice()).collect::<Vec<&mut [f32]>>();
        vibrato.process(ins.as_slice(), outs.as_mut_slice());

        for i in 0..block_size {
            for channel in 0..num_channels {
                assert!(output_block[channel][i].abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn output_equals_delayed_input () {
        let delay_in_secs = 0.0 as f32;
        let f0 = 10.0 as f32;
        let sample_rate_hz = 24000 as usize;
        let num_channels = 1 as usize;

        let mut vibrato = Vibrato::new(sample_rate_hz, num_channels, delay_in_secs);
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
