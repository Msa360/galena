//! Streaming module for SDR and WAV file processing
//!
//! This module provides common functionality for streaming IQ data
//! from different sources (SDR hardware and WAV files) and processing
//! them through demodulation and filtering.

pub mod sdr;
pub mod wav;
pub mod audio_device;

use crate::dsp::{Demodulator, AmDemodulator, FmDemodulator, UsbDemodulator, LsbDemodulator, IirLowPassFilter, process_iq_to_complex};
use crate::app::DemodMode;
use crate::config::{FM_DEMOD_GAIN, SIDEBAND_DEMOD_GAIN};

/// Create the appropriate demodulator based on mode
pub fn create_demodulator(demod_mode: DemodMode, sample_rate: f32) -> Box<dyn Demodulator> {
    match demod_mode {
        DemodMode::FM => Box::new(FmDemodulator::new(sample_rate).with_gain(FM_DEMOD_GAIN)),
        DemodMode::Raw => Box::new(AmDemodulator::new(sample_rate)),
        DemodMode::USB => Box::new(UsbDemodulator::new(sample_rate).with_gain(SIDEBAND_DEMOD_GAIN)),
        DemodMode::LSB => Box::new(LsbDemodulator::new(sample_rate).with_gain(SIDEBAND_DEMOD_GAIN)),
    }
}

/// Demodulate IQ samples and apply filtering/decimation
/// 
/// This function processes raw IQ buffer data through demodulation,
/// applies low-pass filtering, and decimates to the target audio rate.
/// 
/// # Arguments
/// * `buffer` - Raw IQ samples as u8 bytes
/// * `demodulator` - Trait object for demodulation
/// * `lowpass_filter` - Filter to apply to demodulated samples
/// * `decimation` - Decimation factor (e.g., 50 for 2.4MHz -> 48kHz)
pub fn demodulate_and_filter(
    buffer: &[u8],
    demodulator: &mut dyn Demodulator,
    lowpass_filter: &mut IirLowPassFilter,
    decimation: usize,
) -> Vec<f32> {
    let mut audio_samples = Vec::with_capacity(buffer.len() / 2 / decimation);

    // Convert to complex samples
    let complex_samples = process_iq_to_complex(buffer);

    // Demodulate using the trait object
    let demod_output = demodulator.demodulate(&complex_samples);

    // Apply low-pass filter and decimate
    for (i, &sample) in demod_output.iter().enumerate() {
        let filtered = lowpass_filter.process_sample(sample);
        
        // Decimate to audio rate
        if i % decimation == 0 {
            audio_samples.push(filtered);
        }
    }

    audio_samples
}
