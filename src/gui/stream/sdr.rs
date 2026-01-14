//! SDR streaming and processing module

use crate::dsp::{Demodulator, IirLowPassFilter, SpectrumProcessor};
use crate::app::{DemodMode, AudioDevice};
use crate::gui::Message;
use super::{create_demodulator, demodulate_and_filter, audio_device};
use rodio::{OutputStreamBuilder, Sink, buffer::SamplesBuffer};
use tokio::sync::mpsc;
use iced::Subscription;

const SAMPLE_RATE: u32 = 2_400_000;
const AUDIO_RATE: u32 = 48_000;
const DECIMATION: usize = (SAMPLE_RATE / AUDIO_RATE) as usize;
const BUFFER_SIZE: usize = 262144;
const FFT_SIZE: usize = 16384;

/// Create a subscription for SDR streaming
pub fn subscription(
    frequency: u64,
    demod_mode: DemodMode,
    device_index: usize,
    audio_device: Option<AudioDevice>,
) -> Subscription<Message> {
    use iced::futures::SinkExt;

    Subscription::run_with_id(
        (frequency, demod_mode, device_index, audio_device.clone()),
        iced::stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            start_sdr_stream(frequency, demod_mode, device_index, audio_device, tx);

            while let Some(msg) = rx.recv().await {
                let _ = output.send(msg).await;
            }
        })
    )
}

/// Create a subscription to check RTL-SDR connection status without streaming
pub fn connection_check_subscription(device_index: usize) -> Subscription<Message> {
    use iced::futures::SinkExt;
    use std::time::Duration;
    
    Subscription::run_with_id(
        ("sdr_connection_check", device_index),
        iced::stream::channel(1, move |mut output| async move {
            // Check connection status every 2 seconds
            loop {
                let is_connected = rtl_sdr_rs::RtlSdr::open(rtl_sdr_rs::DeviceId::Index(device_index)).is_ok();
                let _ = output.send(Message::SdrConnectionStatus(is_connected)).await;
                
                // Sleep using async-std or smol
                std::thread::sleep(Duration::from_secs(2));
            }
        })
    )
}

/// Start SDR streaming and audio playback
fn start_sdr_stream(
    frequency: u64,
    demod_mode: DemodMode,
    device_index: usize,
    audio_device: Option<AudioDevice>,
    tx: mpsc::Sender<Message>,
) {
    std::thread::spawn(move || {
        if let Err(e) = run_sdr_loop(frequency, demod_mode, device_index, audio_device, tx) {
            eprintln!("SDR thread error: {e:?}");
        }
    });
}

fn run_sdr_loop(
    frequency: u64,
    demod_mode: DemodMode,
    device_index: usize,
    audio_device: Option<AudioDevice>,
    tx: mpsc::Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize audio output with selected device
    let stream = if let Some(device) = audio_device.as_ref().and_then(audio_device::reconstruct_audio_device) {
        OutputStreamBuilder::from_device(device)?
            .open_stream_or_fallback()?
    } else {
        OutputStreamBuilder::from_default_device()?
            .open_stream_or_fallback()?
    };

    let sink = Sink::connect_new(stream.mixer());
    sink.play();

    // Initialize RTL-SDR device
    let device = match open_rtl_sdr(frequency, device_index) {
        Ok(dev) => {
            let _ = tx.blocking_send(Message::SdrConnectionStatus(true));
            dev
        }
        Err(e) => {
            let _ = tx.blocking_send(Message::SdrConnectionStatus(false));
            let _ = tx.blocking_send(Message::Error(e.clone()));
            return Err(e.into());
        }
    };

    // Initialize DSP components
    let mut spectrum_processor = SpectrumProcessor::new();
    let mut demodulator: Box<dyn Demodulator> = create_demodulator(demod_mode, SAMPLE_RATE as f32);
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
                    demodulator.as_mut(),
                    &mut lowpass_filter,
                    DECIMATION,
                );

                // Send audio to output
                let source = SamplesBuffer::new(1, AUDIO_RATE, audio_samples);
                sink.append(source);
            }
            Err(e) => {
                let _ = tx.blocking_send(Message::SdrConnectionStatus(false));
                let _ = tx.blocking_send(Message::Error(format!("Read error: {e:?}")));
                break;
            }
        }
    }

    Ok(())
}

fn open_rtl_sdr(frequency: u64, device_index: usize) -> Result<rtl_sdr_rs::RtlSdr, String> {
    let mut device = rtl_sdr_rs::RtlSdr::open(rtl_sdr_rs::DeviceId::Index(device_index))
        .map_err(|e| format!("Failed to open device: {e:?}"))?;

    device.set_center_freq(frequency as u32)
        .map_err(|e| format!("Failed to set frequency: {e:?}"))?;

    device.set_tuner_gain(rtl_sdr_rs::TunerGain::Manual(300))
        .map_err(|e| format!("Failed to set gain: {e:?}"))?;

    device.set_sample_rate(SAMPLE_RATE)
        .map_err(|e| format!("Failed to set sample rate: {e:?}"))?;

    device.reset_buffer()
        .map_err(|e| format!("Failed to reset buffer: {e:?}"))?;

    Ok(device)
}
