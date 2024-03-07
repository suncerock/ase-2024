use std::{fs::File, io::Write, f32::consts::PI};

mod ring_buffer;
mod vibrato;
mod lfo;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   // Parse command line arguments
   let args: Vec<String> = std::env::args().collect();
   if args.len() > 5 || args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <oscillator_f0> <delay_in_secs>", args[0]);
        return
   }
   let oscillator_f0 = if args.len() <= 3 {5.0} else {args[4].parse::<f32>().unwrap()};
   let delay_in_secs = if args.len() <= 4 {2.0} else {args[5].parse::<f32>().unwrap()};

   // Open the input wave file
   let mut reader = hound::WavReader::open(&args[1]).unwrap();
   let spec = reader.spec();
   let channels = spec.channels;

   // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
   //       Use the following block size:
   let sample_rate = spec.sample_rate;
   let block_size = 1024;

   let input: Vec<Vec<f32>> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect::<Vec<f32>>()
        .chunks(channels as usize)
        .map(|chunk| chunk.to_vec())
        .collect();

   let mut vibrato_process = vibrato::Vibrato::new(sample_rate, channels as usize);
   vibrato_process.set_param(vibrato::VibratoParam::DelayInSecs, delay_in_secs);
   vibrato_process.set_param(vibrato::VibratoParam::OscillatorF0, oscillator_f0);

   let input_slice: Vec<&[f32]> = input.iter().map(|row| row.as_slice()).collect();
   
   let mut writer = hound::WavWriter::create(&args[2], hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: spec.bits_per_sample,
        sample_format: spec.sample_format
   }).unwrap();

   let mut output = vec![vec![0.0 as f32; channels as usize]; block_size];
   let mut output_slice: Vec<&mut [f32]> = output.iter_mut().map(|row| row.as_mut_slice()).collect();
   let num_blocks = input.len() / block_size;
   for i in 0..num_blocks {
        vibrato_process.process(&input_slice[i*block_size..(i+1)*block_size], output_slice.as_mut_slice());
        for j in 0..block_size {
            for c in 0..channels as usize {
                writer.write_sample((output_slice[j][c] * i16::MAX as f32) as i16).unwrap();
            }
        }
   }

}
