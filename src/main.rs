use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::fmt::Write as _;
use std::io::Write as _;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    let args: Vec<String> = std::env::args().collect();
    // TODO: your code here
    let input_path: &String = &args[1];
    let output_path: &String = &args[2];

    // Open the input wave file and determine number of channels
    // TODO: your code here; see `hound::WavReader::open`.
    let mut reader = hound::WavReader::open(input_path).unwrap();
    let channels: u16 = reader.spec().channels;

    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

    // Read audio data and write it to the output text file (one column per channel)
    // TODO: your code here; we suggest using `hound::WavReader::samples`, `File::create`, and `write!`.
    //       Remember to convert the samples to floating point values and respect the number of channels!
    let output_file = File::create(output_path);
    // let mut writer = BufWriter::new(output_file);

    for i in 0..samples.len() {
        // dbg!(samples[i]);
        writeln!(output_file, "{}", samples[i]);
    }
}
