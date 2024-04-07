use std::{fs::File, io::Write};

mod ring_buffer;
mod fast_convolver;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    let sample_rate = spec.sample_rate;

    // Define the block size
    let block_size = 1024;

    // Create the fast convolver
    let convolution_model = fast_convolver::ConvolutionMode::TimeDomain;
    let mut convolver = fast_convolver::FastConvolver::new(&[], convolution_model);


}
