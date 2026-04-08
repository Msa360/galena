/// DSP library for software-defined radio
///
/// This library provides digital signal processing capabilities including
/// filters and demodulation functions.
pub mod filters;
pub mod demodulation;
mod spectrum;

#[cfg(test)]
mod tests;

pub use demodulation::{Demodulator, AmDemodulator, FmDemodulator, UsbDemodulator, LsbDemodulator};
pub use filters::IirLowPassFilter;
pub use spectrum::{SpectrumProcessor, process_iq_to_complex};
