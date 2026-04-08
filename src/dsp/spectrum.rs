/// Spectrum analysis and FFT processing
use rustfft::{FftPlanner, num_complex::Complex32};

/// Process IQ samples into complex numbers
pub fn process_iq_to_complex(buffer: &[u8]) -> Vec<Complex32> {
    buffer.chunks_exact(2)
        .map(|c| {
            let i_val = (c[0] as f32 - 127.0) / 127.0;
            let q_val = (c[1] as f32 - 127.0) / 127.0;
            Complex32::new(i_val, q_val)
        })
        .collect()
}

/// Spectrum processor for FFT analysis
pub struct SpectrumProcessor {
    planner: FftPlanner<f32>,
}

impl SpectrumProcessor {
    pub fn new() -> Self {
        Self {
            planner: FftPlanner::new(),
        }
    }

    /// Process IQ buffer into FFT spectrum data
    /// Returns magnitude spectrum in dB, FFT-shifted for display
    pub fn process_fft(&mut self, buffer: &[u8]) -> Vec<u8> {
        let len = buffer.len() / 2;
        let fft = self.planner.plan_fft_forward(len);
        
        let mut input = process_iq_to_complex(buffer);
        
        // Apply Hann window
        self.apply_hann_window(&mut input);

        // Perform FFT
        fft.process(&mut input);

        // Convert to magnitude and FFT shift
        self.magnitude_to_db_shifted(&input)
    }

    /// Apply Hann window to reduce spectral leakage
    fn apply_hann_window(&self, samples: &mut [Complex32]) {
        let len = samples.len();
        for (i, val) in samples.iter_mut().enumerate() {
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (len as f32 - 1.0)).cos());
            *val *= window;
        }
    }

    /// Convert FFT output to dB magnitude with FFT shift
    fn magnitude_to_db_shifted(&self, fft_output: &[Complex32]) -> Vec<u8> {
        let len = fft_output.len();
        let half = len / 2;
        let mut output = vec![0u8; len];

        for (i, val) in fft_output.iter().enumerate() {
            let mag = val.norm();
            let db = 20.0 * mag.log10();
            let scaled = ((db + 40.0) * 4.0).clamp(0.0, 255.0) as u8;

            // FFT shift: swap halves
            let target_idx = if i < half { i + half } else { i - half };
            output[target_idx] = scaled;
        }

        output
    }
}

impl Default for SpectrumProcessor {
    fn default() -> Self {
        Self::new()
    }
}
