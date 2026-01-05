/// SDR streaming and processing module

use crate::dsp::{AmDemodulator, FmDemodulator, IirLowPassFilter, SpectrumProcessor, process_iq_to_complex};
use crate::app::{Message, DemodMode};
use rodio::{OutputStreamBuilder, Sink, buffer::SamplesBuffer};
use tokio::sync::mpsc;

const SAMPLE_RATE: u32 = 2_400_000;
const AUDIO_RATE: u32 = 48_000;
const DECIMATION: usize = (SAMPLE_RATE / AUDIO_RATE) as usize;
const BUFFER_SIZE: usize = 262144;
const FFT_SIZE: usize = 16384;

/// Start SDR streaming and audio playback
pub fn start_sdr_stream(frequency: u64, demod_mode: DemodMode, tx: mpsc::Sender<Message>) {
    std::thread::spawn(move || {
        if let Err(e) = run_sdr_loop(frequency, demod_mode, tx) {
            eprintln!("SDR thread error: {:?}", e);
        }
    });
}

fn run_sdr_loop(
    frequency: u64,
    demod_mode: DemodMode,
    tx: mpsc::Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize audio output
    let stream = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream.mixer());
    sink.play();

    // Initialize RTL-SDR device
    let device = open_rtl_sdr(frequency)?;

    // Initialize DSP components
    let mut spectrum_processor = SpectrumProcessor::new();
    let mut fm_demod = FmDemodulator::new(SAMPLE_RATE as f32).with_gain(0.8);
    let am_demod = AmDemodulator::new(SAMPLE_RATE as f32);
    let mut lowpass_filter = IirLowPassFilter::new(10_000.0, SAMPLE_RATE as f32);

    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        if tx.is_closed() {
            break;
        }

        // Read samples from RTL-SDR
        match device.read_sync(&mut buffer) {
            Ok(_) => {
                // Process FFT for UI (use subset for performance)
                let spectrum = spectrum_processor.process_fft(&buffer[0..FFT_SIZE]);
                if tx.blocking_send(Message::SpectrumData(spectrum)).is_err() {
                    break;
                }

                // Demodulate and create audio
                let audio_samples = demodulate_and_filter(
                    &buffer,
                    demod_mode,
                    &mut fm_demod,
                    &am_demod,
                    &mut lowpass_filter,
                );

                // Send audio to output
                let source = SamplesBuffer::new(1, AUDIO_RATE, audio_samples);
                sink.append(source);
            }
            Err(e) => {
                let _ = tx.blocking_send(Message::Error(format!("Read error: {:?}", e)));
                break;
            }
        }
    }

    Ok(())
}

fn open_rtl_sdr(frequency: u64) -> Result<rtl_sdr_rs::RtlSdr, String> {
    let mut device = rtl_sdr_rs::RtlSdr::open(rtl_sdr_rs::DeviceId::Index(0))
        .map_err(|e| format!("Failed to open device: {:?}", e))?;

    device.set_center_freq(frequency as u32)
        .map_err(|e| format!("Failed to set frequency: {:?}", e))?;
    
    device.set_tuner_gain(rtl_sdr_rs::TunerGain::Manual(300))
        .map_err(|e| format!("Failed to set gain: {:?}", e))?;
    
    device.set_sample_rate(SAMPLE_RATE)
        .map_err(|e| format!("Failed to set sample rate: {:?}", e))?;
    
    device.reset_buffer()
        .map_err(|e| format!("Failed to reset buffer: {:?}", e))?;

    Ok(device)
}

fn demodulate_and_filter(
    buffer: &[u8],
    demod_mode: DemodMode,
    fm_demod: &mut FmDemodulator,
    am_demod: &AmDemodulator,
    lowpass_filter: &mut IirLowPassFilter,
) -> Vec<f32> {
    let mut audio_samples = Vec::with_capacity(buffer.len() / 2 / DECIMATION);

    // Convert to complex samples
    let complex_samples = process_iq_to_complex(buffer);

    // Demodulate based on mode
    let demod_output = match demod_mode {
        DemodMode::FM => fm_demod.demodulate(&complex_samples),
        DemodMode::Raw => am_demod.demodulate(&complex_samples),
    };

    // Apply low-pass filter and decimate
    for (i, &sample) in demod_output.iter().enumerate() {
        let filtered = lowpass_filter.process_sample(sample);
        
        // Decimate to audio rate
        if i % DECIMATION == 0 {
            audio_samples.push(filtered);
        }
    }

    audio_samples
}
