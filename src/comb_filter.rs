pub struct CombFilter {
    // TODO: your code here
    filter_type: FilterType,
    gain: f32,
    max_delay_secs: f32,
    sample_rate_hz: f32,
    max_delay_samples: usize,
    num_channels: usize
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        CombFilter {
            filter_type: filter_type,
            gain: 0.0,
            max_delay_secs: max_delay_secs,
            sample_rate_hz: sample_rate_hz,
            max_delay_samples: (max_delay_secs * sample_rate_hz) as usize,
            num_channels: num_channels
        }
    }

    pub fn reset(&mut self) {
        self.gain = 0.0;
        self.max_delay_secs = 0.0;
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let mut delay_line = vec![vec![0.0 as f32; self.num_channels]; self.max_delay_samples];
        for i in 0..input.len() {
            for channel in 0..self.num_channels {
                output[i][channel] = input[i][channel] + self.gain * delay_line[self.max_delay_samples - 1][channel];
                for delay_index in (1..self.max_delay_samples).rev() {
                    delay_line[delay_index][channel] = delay_line[delay_index - 1][channel];
                }
                delay_line[0][channel] = match self.filter_type {
                    FilterType::FIR => input[i][channel],
                    FilterType::IIR => output[i][channel]
                }
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => self.set_gain(value),
            FilterParam::Delay => self.set_delay(value),
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.max_delay_secs,
        }
    }

    // TODO: feel free to define other functions for your own use
    fn set_gain(&mut self, value: f32) -> Result<(), Error> {
        self.gain = value;
        Ok(())
    }

    fn set_delay(&mut self, value: f32) -> Result<(), Error> {
        self.max_delay_secs = value;
        Ok(())
    }

}

// TODO: feel free to define other types (here or in other modules) for your own use
