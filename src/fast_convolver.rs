use crate::ring_buffer::RingBuffer;

pub struct FastConvolver {
    // TODO: your fields here
    impulse_response: Vec<f32>,
    mode: ConvolutionMode,
    buffer: RingBuffer<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        FastConvolver {
            impulse_response: impulse_response.to_vec(),
            mode: mode,
            buffer: RingBuffer::new(impulse_response.len() - 1),
        }
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for i in 0..output.len() {
            if self.buffer.len() == 0 {break;}
            output[i] = self.buffer.pop();
        }

        match self.mode {
            ConvolutionMode::TimeDomain => { self.time_domain_process(input, output); }
            ConvolutionMode::FrequencyDomain { block_size } => { self.frequency_domain_process(input, output); }
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        for i in 0..(self.impulse_response.len()-1) {
            output[i] = self.buffer.pop();
        }
    }

    // TODO: feel free to define other functions for your own use
    fn time_domain_process(&mut self, input: &[f32], output: &mut [f32]) {
        // Convolution
        for i in 0..output.len() {
            for j in 0..self.impulse_response.len() {
                output[i] = output[i] + input[i - j] * self.impulse_response[j];
                if i == j {break;}
            }
        }

        // Reverb tail
        for i in 1..self.impulse_response.len() {
            let mut value = self.buffer.pop();
            for j in i..self.impulse_response.len() {
                value = value + self.impulse_response[j] * input[input.len() + i - j - 1];
                if input.len() + i - j - 1 == 0 {break;}
            }
            self.buffer.push(value);
        }
    }

    fn frequency_domain_process(&mut self, input: &[f32], output: &mut [f32]) {
        todo!("frequency_domain_process");
    }
}

// TODO: feel free to define other types (here or in other modules) for your own use
#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;

    #[test]
    fn test_time_domain_convolver () {
        let input_len = 16;
        let ir_len = 4;

        let input = vec![1.0; input_len];
        let impulse_response = vec![1.0; ir_len];

        let mut output = vec![0.0; input_len + ir_len - 1];
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

        convolver.process(&input, &mut output[0..input_len]);
        convolver.flush(&mut output[input_len..]);

        let expected_output = vec![
            1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0,
            4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0,
            3.0, 2.0, 1.0];
        for i in 0..output.len() {
            assert_eq!(output[i], expected_output[i]);
        }
    }

    #[test]
    fn test_time_domain_block () {
        let input_len = 16;
        let ir_len = 4;

        let input = vec![1.0; input_len];
        let impulse_response = vec![1.0; ir_len];

        let mut output = vec![0.0; input_len + ir_len - 1];
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

        let num_blocks = (input_len + ir_len - 1) / ir_len;
        for i in 0..num_blocks {
            let start = i * ir_len;
            let end = (i + 1) * ir_len;
            convolver.process(&input[start..end], &mut output[start..end]);
        }
        convolver.flush(&mut output[input_len..]);

        let expected_output = vec![
            1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0,
            4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0,
            3.0, 2.0, 1.0];
        for i in 0..output.len() {
            assert_eq!(output[i], expected_output[i]);
        }
    }

    #[test]
    fn test_identity () {
        // Generate a random impulse response of 51 samples
        let mut impulse_response: Vec<f32> = vec![0.0; 8];
        let mut rng = rand::thread_rng();
        for i in 0..impulse_response.len() {
            impulse_response[i] = rng.gen_range(-1.0..1.0);
        }

        // Generate an input impulse at sample index 3
        let input: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0, 0.0];
        let mut output: Vec<f32> = vec![0.0; input.len() + impulse_response.len() - 1];

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input, &mut output[..input.len()]);
        convolver.flush(&mut output[input.len()..]);

        // The output should be the impulse response itself, shifted by 3 samples
        for i in 0..3 {
            assert_eq!(output[i], 0.0);
        }
        for i in 3..output.len()-1 {
            assert!((output[i] - impulse_response[i - 3]).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_identity_2 () {
        // Generate a random impulse response of 51 samples
        let mut input: Vec<f32> = vec![0.0; 8];
        let mut rng = rand::thread_rng();
        for i in 0..input.len() {
            input[i] = i as f32;
        }

        // Generate an input impulse at sample index 3
        let impulse_response: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0, 0.0];
        let mut output: Vec<f32> = vec![0.0; input.len() + impulse_response.len() - 1];

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input, &mut output[..input.len()]);
        convolver.flush(&mut output[input.len()..]);

        // The output should be the impulse response itself, shifted by 3 samples
        for i in 0..3 {
            assert_eq!(output[i], 0.0);
        }
        for i in 3..output.len()-1 {
            assert!((output[i] - input[i - 3]).abs() <= f32::EPSILON);
        }
    }
}