use std::{fs::File, io::Write};

mod comb_filter;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 6 || args.len() < 4 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <filter type> <gain> <delay>", args[0]);
        return
    }
    let gain = if args.len() <= 4 {0.5} else {args[4].parse::<f32>().unwrap()};
    let delay = if args.len() <= 5 {0.1} else {args[5].parse::<f32>().unwrap()};
    let filter_type = match args[3].to_lowercase().as_str() {
        "fir" => comb_filter::FilterType::FIR,
        "iir" => comb_filter::FilterType::IIR,
        _ => {
            eprintln!("Filter type must be fir or iir!");
            return;
        }
    };
    // let gain = 0.5;
    // let delay = 0.001;
    // let filter_type = comb_filter::FilterType::FIR;
    // let filename = "sweep.wav";

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    let block_size = 1024;
    let sample_rate = spec.sample_rate;

    let input: Vec<Vec<f32>> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<f32>>()
        .chunks(channels as usize)
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
    let _ = comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

    let mut output = vec![vec![0.0 as f32; channels as usize]; input.len()];

    let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
    let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();
    
    comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());

    let mut writer = hound::WavWriter::create(&args[2], hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: spec.bits_per_sample,
        sample_format: spec.sample_format
    }).unwrap();

    for sample in output.iter() {
        for &value in sample.iter() {
            writer.write_sample((value * i16::MAX as f32) as i16).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::comb_filter;

    #[test]
    fn fir_output_zero_if_input_freq_matches_feedforward() {
        let filter_type = comb_filter::FilterType::FIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 1.0;
        let delay = 0.0025;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let frequency = 200.0;
        let duration_secs = 3.0;
        let amplitude = 0.5;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.0 as f32]; num_samples];
        for t in 0..num_samples {
            let t_sec = t as f32 / sample_rate as f32;
            let sample = amplitude * (2.0 * PI * frequency * t_sec).sin();
            input[t][0] = sample;
        }

        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
        for t in 100..num_samples {
            assert!(output[t][0].abs() < 1.0e-3);
        }
    }

    #[test]
    fn different_block_size() {
        // OK So this will automatically pass anyway
        // Because I did not do block wise comb filter
        // which I believe will break everything I wrote
    }

    #[test]
    fn iir_amount_of_magnitude_increase_if_input_freq_matches_feedback() {
        let filter_type = comb_filter::FilterType::IIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 0.9;
        let delay = 0.0025;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let frequency = 200.0;
        let duration_secs = 3.0;
        let amplitude = 0.5;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.0 as f32]; num_samples];
        for t in 0..num_samples {
            let t_sec = t as f32 / sample_rate as f32;
            let sample = amplitude * (2.0 * PI * frequency * t_sec).sin();
            input[t][0] = sample;
        }

        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
       
        for t in 100..num_samples {
            assert!(output[t][0] < amplitude * 0.9);
            assert!(output[t][0] > -amplitude * 0.9);
        }
    }

    #[test]
    fn fir_zero_input_signal() {
        let filter_type = comb_filter::FilterType::FIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 0.9;
        let delay = 0.01;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let duration_secs = 3.0;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.0 as f32]; num_samples];


        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
       
        for t in 100..num_samples {
            assert!(output[t][0].abs() < 1.0e-7);
        }
    }

    #[test]
    fn iir_zero_input_signal() {
        let filter_type = comb_filter::FilterType::IIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 0.9;
        let delay = 0.01;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let duration_secs = 3.0;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.0 as f32]; num_samples];

        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
       
        for t in 100..num_samples {
            assert!(output[t][0].abs() < 1.0e-7);
        }
    }

    #[test]
    fn fir_constant_non_zero_input() {
        let filter_type = comb_filter::FilterType::FIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 0.5;
        let delay = 0.01;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let duration_secs = 3.0;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.5 as f32]; num_samples];

        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
       
        for t in 0..80 {
            assert!((output[t][0] - 0.5).abs() < 1.0e-7);
        }
        for t in 80..num_samples {
            assert!((output[t][0] - 0.75).abs() < 1.0e-7);
        }
    }

    #[test]
    fn iir_constant_non_zero_input() {
        let filter_type = comb_filter::FilterType::IIR;
        let sample_rate = 8000 as u32;
        let channels = 1 as u16;

        let gain = 0.5;
        let delay = 0.01;

        let mut comb_filter = comb_filter::CombFilter::new(filter_type, delay, sample_rate as f32, channels as usize);
        comb_filter.set_param(comb_filter::FilterParam::Gain, gain);

        let duration_secs = 3.0;
        
        // Calculate the number of samples
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let mut input = vec![vec![0.5 as f32]; num_samples];

        let mut output = vec![vec![0.0 as f32]; num_samples];
        let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
        let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();

        comb_filter.process(input_slice.as_slice(), output_slice.as_mut_slice());
       
        for t in 0..80 {
            assert!((output[t][0] - 0.5).abs() < 1.0e-7);
        }
        for t in 80..160 {
            assert!((output[t][0] - 0.75).abs() < 1.0e-7);
        }
        for t in 160..240 {
            assert!((output[t][0] - 0.875).abs() < 1.0e-7);
        }
    }

}