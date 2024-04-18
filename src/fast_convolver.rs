use crate::ring_buffer::RingBuffer;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc; // Make sure Arc is imported

pub struct FastConvolver {
    // TODO: your fields here
    impulse_response: Vec<f32>,
    mode: ConvolutionMode,
    buffer: Vec<f32>,
    ir_blocks: Vec<Vec<Complex<f32>>>,
    overlap_buffer: Vec<f32>,
    block_size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let block_size = match mode {
            ConvolutionMode::FrequencyDomain { block_size } => block_size,
            _ => panic!("Block size must be specified for FrequencyDomain mode"),
        };
        
        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(block_size);

        // Pass the fft Arc directly
        let ir_blocks = Self::partition_and_transform_ir(impulse_response, fft, block_size);

        FastConvolver {
            impulse_response: impulse_response.to_vec(),
            mode,
            buffer: vec![0.0; block_size],
            ir_blocks,
            overlap_buffer: vec![0.0; 2 * block_size],
            block_size,
        }
    }
    pub fn partition_and_transform_ir(ir: &[f32], fft: Arc<dyn Fft<f32>>, block_size: usize) -> Vec<Vec<Complex<f32>>> {
        ir.chunks(block_size)
            .map(|chunk| {
                let mut input = vec![Complex::new(0.0, 0.0); block_size];
                input.iter_mut().zip(chunk).for_each(|(a, &b)| *a = Complex::new(b, 0.0));
                // Use the fft plan directly
                fft.process(&mut input);
                input
            })
            .collect()
    }
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.mode {
            ConvolutionMode::TimeDomain => self.time_domain_process(input, output),
            ConvolutionMode::FrequencyDomain { block_size: _ } => self.frequency_domain_process(input, output),
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        for i in 0..(self.impulse_response.len()-1) {
            output[i] = self.buffer[i];
        }
    }

    fn time_domain_process(&mut self, input: &[f32], output: &mut [f32]) {
        let mut full_output = vec![0.0; input.len() + self.impulse_response.len() - 1];

        // Convolution
        for i in 0..full_output.len() {
            for j in 0..self.impulse_response.len() {
                if i >= input.len() + j {continue;}
                full_output[i] = full_output[i] + input[i - j] * self.impulse_response[j];
                if i == j {break;}
            }
        }

        // Overlap add
        for i in 0..self.impulse_response.len()-1 {
            full_output[i] = self.buffer[i] + full_output[i];
        }

        // Output
        for i in 0..input.len() {
            output[i] = full_output[i];
        }

        // Update buffer
        for i in 0..self.impulse_response.len()-1 {
            self.buffer[i] = full_output[input.len() + i];
        }

    }

    fn frequency_domain_process(&mut self, input: &[f32], output: &mut [f32]) {
        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(self.block_size);
        let ifft = fft_planner.plan_fft_inverse(self.block_size);
    
        // Clear the overlap buffer
        self.overlap_buffer.fill(0.0);
    
        for (i, chunk) in input.chunks(self.block_size).enumerate() {
            let mut input_block = vec![Complex::new(0.0, 0.0); self.block_size];
            input_block.iter_mut().zip(chunk).for_each(|(a, &b)| *a = Complex::new(b, 0.0));
            fft.process(&mut input_block);
    
            let mut output_block = vec![Complex::new(0.0, 0.0); self.block_size];
            for (j, (input_value, ir_value)) in input_block.iter().zip(self.ir_blocks.get(i).unwrap_or(&vec![Complex::new(0.0, 0.0); self.block_size]).iter()).enumerate() {
                output_block[j] = *input_value * *ir_value;
            }
    
            ifft.process(&mut output_block);
    
            for (j, &complex) in output_block.iter().enumerate() {
                let index = i * self.block_size + j;
                let buffer_len = self.overlap_buffer.len();  // Store buffer length
                self.overlap_buffer[index % buffer_len] += complex.re; // We only need the real part
            }
        }
    
        // Copy from overlap buffer to output
        output.copy_from_slice(&self.overlap_buffer[..input.len()]);
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
        let mut impulse_response: Vec<f32> = vec![0.0; 51];
        let mut rng = rand::thread_rng();
        for i in 0..impulse_response.len() {
            impulse_response[i] = rng.gen_range(-1.0..1.0);
        }

        // Generate an input impulse at sample index 3
        let input: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output: Vec<f32> = vec![0.0; input.len() + impulse_response.len() - 1];

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input, &mut output[..input.len()]);
        convolver.flush(&mut output[input.len()..]);

        // The output should be the impulse response itself, shifted by 3 samples
        // The flush is tested together with the process
        for i in 0..3 {
            assert_eq!(output[i], 0.0);
        }
        for i in 3..output.len()-6 {
            assert!((output[i] - impulse_response[i - 3]).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_block_size () {
        // Generate a random impulse response of 51 samples
        let mut input: Vec<f32> = vec![0.0; 10000];
        let mut rng = rand::thread_rng();
        for i in 0..input.len() {
            input[i] = rng.gen_range(-1.0..1.0);
        }

        // Generate an input impulse at sample index 3
        let impulse_response: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0];
        let mut output: Vec<f32> = vec![0.0; input.len() + impulse_response.len() - 1];

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input[0..1], &mut output[0..1]);
        convolver.process(&input[1..14], &mut output[1..14]);
        convolver.process(&input[14..1037], &mut output[14..1037]);
        convolver.process(&input[1037..3085], &mut output[1037..3085]);
        convolver.process(&input[3085..3086], &mut output[3085..3086]);
        convolver.process(&input[3086..3103], &mut output[3086..3103]);
        convolver.process(&input[3103..8103], &mut output[3103..8103]);
        convolver.process(&input[8103..10000], &mut output[8103..10000]);
        convolver.flush(&mut output[10000..]);

        // The output should be the impulse response itself, shifted by 3 samples
        // The flush is tested together with the process
        for i in 0..3 {
            assert_eq!(output[i], 0.0);
        }
        for i in 3..output.len() {
            assert!((output[i] - input[i - 3]).abs() <= f32::EPSILON);
        }
    }

    #[test]
    fn test_frequency_domain_convolver_basic() {
        let input_len = 16;
        let ir_len = 4;
        let block_size = 4; // Test with a specific block size

        let input = vec![1.0; input_len];
        let impulse_response = vec![1.0; ir_len];
        let mut output = vec![0.0; input_len]; // Output size adjusted for no overflow beyond input length

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::FrequencyDomain { block_size });
        convolver.process(&input, &mut output);

        // Expected output calculated manually or from a known good implementation
        let expected_output = vec![1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0, 4.0];

        assert_eq!(output, expected_output, "Outputs do not match for basic frequency domain convolution.");
    }

    #[test]
    fn test_frequency_domain_convolver_latency_compensation() {
        let input_len = 1024;
        let ir_len = 64;
        let block_size = 256; // Larger block size

        let input = vec![0.0; input_len];
        input[3] = 1.0; // Impulse at position 3
        let impulse_response = vec![0.5; ir_len]; // Some non-trivial impulse response
        let mut output = vec![0.0; input_len];

        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::FrequencyDomain { block_size });
        convolver.process(&input, &mut output);

        // Check for latency and correct output
        // Assuming the first non-zero output should start at the position of the impulse + some expected latency
        let expected_start = 3; // Adjust based on the observed latency
        let first_non_zero = output.iter().position(|&x| x != 0.0).unwrap();

        assert_eq!(first_non_zero, expected_start, "Latency compensation is incorrect.");
    }
}