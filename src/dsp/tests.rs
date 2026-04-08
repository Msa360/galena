#[cfg(test)]
mod tests {
    use crate::dsp::filters::IirLowPassFilter;
    use crate::dsp::demodulation::{AmDemodulator, FmDemodulator, Demodulator};
    use rustfft::num_complex::Complex32;

    #[test]
    fn test_iir_lowpass_filter_initialization() {
        let mut filter = IirLowPassFilter::new(1000.0, 48000.0);
        // Filter should be initialized without panicking
        let output = filter.process_sample(0.5);
        assert!(output.is_finite());
    }

    #[test]
    fn test_iir_lowpass_filter_dc_blocking() {
        let mut filter = IirLowPassFilter::new(1000.0, 48000.0);
        // Apply constant signal
        let mut outputs = Vec::new();
        for _ in 0..100 {
            outputs.push(filter.process_sample(1.0));
        }
        // After convergence, output should be close to input
        let final_output = outputs[outputs.len() - 1];
        assert!((final_output - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_am_demodulator() {
        let mut demod = AmDemodulator::new(48000.0);
        let samples = vec![
            Complex32::new(1.0, 0.0),
            Complex32::new(0.0, 1.0),
            Complex32::new(-1.0, 0.0),
        ];
        let output = demod.demodulate(&samples);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], 1.0); // |1+0j| = 1.0
        assert_eq!(output[1], 1.0); // |0+1j| = 1.0
        assert_eq!(output[2], 1.0); // |-1+0j| = 1.0
    }

    #[test]
    fn test_fm_demodulator_state() {
        let mut demod = FmDemodulator::new(48000.0).with_gain(1.0);
        let samples = vec![
            Complex32::new(1.0, 0.0),
            Complex32::new(1.0, 0.0),
        ];
        // Should not panic and return same length
        let output = demod.demodulate(&samples);
        assert_eq!(output.len(), 2);
        assert!(output.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn test_fm_demodulator_reset() {
        let mut demod = FmDemodulator::new(48000.0).with_gain(1.0);
        demod.reset();
        // Should not panic
        let samples = vec![Complex32::new(1.0, 0.0)];
        let _ = demod.demodulate(&samples);
    }
}
