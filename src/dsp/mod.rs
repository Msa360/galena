/// DSP library for software-defined radio
/// 
/// This library provides digital signal processing capabilities including
/// filters and demodulation functions.

pub mod filters;
pub mod demod;
pub mod sdr_stream;
pub mod wav_stream;
mod spectrum;

pub use demod::{AmDemodulator, FmDemodulator};
pub use filters::IirLowPassFilter;
pub use spectrum::{SpectrumProcessor, process_iq_to_complex};
