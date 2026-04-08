//! Configuration constants for the SDR application

/// Sample rate from RTL-SDR devices (2.4 MHz)
pub const SDR_SAMPLE_RATE: u32 = 2_400_000;

/// Audio output rate (48 kHz)
pub const AUDIO_RATE: u32 = 48_000;

/// Decimation factor: SDR_SAMPLE_RATE / AUDIO_RATE
pub const DECIMATION: usize = (SDR_SAMPLE_RATE as usize) / (AUDIO_RATE as usize);

/// Buffer size for reading from SDR device
pub const SDR_BUFFER_SIZE: usize = 262144;

/// FFT size for spectrum analysis
pub const FFT_SIZE: usize = 16384;

/// Chunk size for WAV file processing
pub const WAV_CHUNK_SIZE: usize = 262144;

/// Low-pass filter cutoff frequency (Hz)
pub const LOWPASS_CUTOFF_HZ: f32 = 10_000.0;

/// FM demodulator gain
pub const FM_DEMOD_GAIN: f32 = 0.8;

/// USB/LSB demodulator gain
pub const SIDEBAND_DEMOD_GAIN: f32 = 2.0;

/// RTL-SDR tuner gain (manual)
pub const RTL_TUNER_GAIN: i32 = 300;

/// Maximum waterfall history lines
pub const MAX_WATERFALL_LINES: usize = 100;

/// Connection check interval (seconds)
pub const CONNECTION_CHECK_INTERVAL_SECS: u64 = 2;
