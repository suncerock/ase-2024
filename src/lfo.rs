use std::f32::consts::PI;


pub struct WavetableLFO {
    table: Vec<f32>,
    phase: f32,
    phase_increment: f32,
}

impl WavetableLFO {
    pub fn new(table_size: usize, frequency: f32, sample_rate: usize) -> Self {
        let phase_increment = frequency / sample_rate as f32;
        let mut sine_wave: Vec<f32> = Vec::with_capacity(table_size);
        for i in 0..table_size {
            let phase = i as f32 / table_size as f32;
            sine_wave.push((2.0 as f32 * PI * phase).sin());
        }
        WavetableLFO {
            table: sine_wave.clone(),
            phase: 0.0,
            phase_increment,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let index = (self.phase * self.table.len() as f32) as usize;
        let sample = self.table[index % self.table.len()];
        
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavetable() {
        // Create a sine wave table
        let table_size = 100 as usize;
        let sample_rate = 44100 as usize;
        let frequency = 10.0; // Hz, adjust as needed
        let mut lfo = WavetableLFO::new(table_size, frequency, sample_rate);

        // Generate and print some samples
        for _ in 0..100 {
            dbg!(lfo.next_sample());
        }
    }
}
