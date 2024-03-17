use std::{fs::File};

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
   let channels = spec.channels as usize;

   // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
   //       Use the following block size:
   let sample_rate = spec.sample_rate as usize;
   let block_size = 1024;

   let out = File::create(&args[2]).expect("Unable to create file");
   let mut writer = hound::WavWriter::new(out, spec).unwrap();

   // Read audio data and write it to the output text file (one column per channel)
   let mut block = vec![Vec::<f32>::with_capacity(block_size); channels];
   let mut output_block = vec![vec![0.0_f32; block_size]; channels];
   let num_samples = reader.len() as usize;

   let mut vibrato_process = vibrato::Vibrato::new(sample_rate, channels, delay_in_secs);
   vibrato_process.set_param(vibrato::VibratoParam::DelayInSecs, delay_in_secs);
   vibrato_process.set_param(vibrato::VibratoParam::OscillatorF0, oscillator_f0);

   for (i, sample) in reader.samples::<i16>().enumerate() {
     let sample = sample.unwrap() as f32 / (1 << 15) as f32;
     block[i % channels].push(sample);
     if (i % (channels * 1024) == 0) || (i == num_samples - 1) {
         // Process block
         let ins = block.iter().map(|c| c.as_slice()).collect::<Vec<&[f32]>>();
         let mut outs = output_block.iter_mut().map(|c| c.as_mut_slice()).collect::<Vec<&mut [f32]>>();
         vibrato_process.process(ins.as_slice(), outs.as_mut_slice());
         for j in 0..(channels * block[0].len()) {
             writer.write_sample((output_block[j % channels][j / channels] * (1 << 15) as f32) as i32).unwrap();
         }
         for channel in block.iter_mut() {
             channel.clear();
         }
     }
}

}
