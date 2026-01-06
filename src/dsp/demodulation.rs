/// Demodulation functions for various modulation schemes

use rustfft::num_complex::Complex32;

/// Common trait for all demodulators
pub trait Demodulator {
    /// Demodulate complex IQ samples to audio samples
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32>;
    
    /// Reset demodulator state (if applicable)
    fn reset(&mut self) {
        // Default implementation does nothing
    }
}

/// Amplitude Modulation (AM) demodulator
pub struct AmDemodulator {
    #[allow(dead_code)]
    sample_rate: f32,
}

impl AmDemodulator {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }
}

impl Demodulator for AmDemodulator {
    /// Demodulate complex IQ samples using envelope detection
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32> {
        samples.iter().map(|s| s.norm()).collect()
    }
}

/// Frequency Modulation (FM) demodulator using phase discrimination
pub struct FmDemodulator {
    #[allow(dead_code)]
    sample_rate: f32,
    prev_sample: Complex32,
    gain: f32,
}

impl FmDemodulator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            prev_sample: Complex32::new(0.0, 0.0),
            gain: 0.8,
        }
    }

    /// Set the output gain
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }
}

impl Demodulator for FmDemodulator {
    /// Demodulate complex IQ samples using phase differentiation
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32> {
        samples.iter().map(|&curr| {
            let conj = self.prev_sample.conj();
            let prod = curr * conj;
            let angle = prod.arg();
            self.prev_sample = curr;
            angle * self.gain
        }).collect()
    }

    /// Reset demodulator state
    fn reset(&mut self) {
        self.prev_sample = Complex32::new(0.0, 0.0);
    }
}

/// Phase Modulation (PM) demodulator
pub struct PmDemodulator {
    #[allow(dead_code)]
    sample_rate: f32,
    prev_phase: f32,
    gain: f32,
}

impl PmDemodulator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            prev_phase: 0.0,
            gain: 1.0,
        }
    }

    /// Set the output gain
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }
}

impl Demodulator for PmDemodulator {
    /// Demodulate complex IQ samples by extracting instantaneous phase
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32> {
        samples.iter().map(|&sample| {
            let phase = sample.arg();
            
            // Unwrap phase discontinuities
            let mut delta = phase - self.prev_phase;
            
            // Handle phase wrap-around (-π to π)
            if delta > std::f32::consts::PI {
                delta -= 2.0 * std::f32::consts::PI;
            } else if delta < -std::f32::consts::PI {
                delta += 2.0 * std::f32::consts::PI;
            }
            
            self.prev_phase = phase;
            delta * self.gain
        }).collect()
    }

    /// Reset demodulator state
    fn reset(&mut self) {
        self.prev_phase = 0.0;
    }
}

/// Upper Sideband (USB) demodulator
pub struct UsbDemodulator {
    #[allow(dead_code)]
    sample_rate: f32,
    gain: f32,
}

impl UsbDemodulator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            gain: 1.0,
        }
    }

    /// Set the output gain
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }
}

impl Demodulator for UsbDemodulator {
    /// Demodulate complex IQ samples for USB by taking the real component
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32> {
        samples.iter().map(|s| s.re * self.gain).collect()
    }
}

/// Lower Sideband (LSB) demodulator
pub struct LsbDemodulator {
    #[allow(dead_code)]
    sample_rate: f32,
    gain: f32,
}

impl LsbDemodulator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            gain: 1.0,
        }
    }

    /// Set the output gain
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }
}

impl Demodulator for LsbDemodulator {
    /// Demodulate complex IQ samples for LSB by taking the imaginary component
    fn demodulate(&mut self, samples: &[Complex32]) -> Vec<f32> {
        samples.iter().map(|s| s.im * self.gain).collect()
    }
}
