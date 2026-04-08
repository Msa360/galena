//! WAV file streaming and processing module

use crate::dsp::{Demodulator, IirLowPassFilter, SpectrumProcessor};
use crate::app::{DemodMode, AudioDevice};
use crate::gui::Message;
use crate::config::*;
use super::{create_demodulator, demodulate_and_filter, audio_device};
use rodio::{OutputStreamBuilder, Sink, buffer::SamplesBuffer};
use tokio::sync::mpsc;
use std::path::Path;
use iced::Subscription;

/// Create a subscription for WAV file streaming
pub fn subscription(
    file_path: String,
    demod_mode: DemodMode,
    start_position: usize,
    is_playing: bool,
    audio_device: Option<AudioDevice>,
) -> Subscription<Message> {
    use iced::futures::SinkExt;

    Subscription::run_with_id(
        (file_path.clone(), demod_mode, is_playing, audio_device.clone()),  // Include audio_device in ID
        iced::stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // Only start the stream thread if we're playing
            if is_playing {
                start_wav_stream(file_path, demod_mode, start_position, audio_device, tx);

                while let Some(msg) = rx.recv().await {
                    let _ = output.send(msg).await;
                }
            }
        })
    )
}

/// Start WAV file streaming and audio playback
fn start_wav_stream(
    file_path: String,
    demod_mode: DemodMode,
    start_position: usize,
    audio_device: Option<AudioDevice>,
    tx: mpsc::Sender<Message>,
) {
    std::thread::spawn(move || {
        if let Err(e) = run_wav_loop(&file_path, demod_mode, start_position, audio_device, tx) {
            log::error!("WAV thread error: {e:?}");
        }
    });
}

fn run_wav_loop(
    file_path: &str,
    demod_mode: DemodMode,
    start_position: usize,
    audio_device: Option<AudioDevice>,
    tx: mpsc::Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open WAV file
    let mut reader = hound::WavReader::open(Path::new(file_path))
        .map_err(|e| format!("Failed to open WAV file: {e:?}"))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    // Validate WAV format (expecting I/Q data as interleaved samples)
    if spec.channels != 2 {
        return Err("WAV file must have 2 channels (I/Q data)".into());
    }

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

    // Keep stream alive by holding onto it
    let _stream_handle = stream;

    // Initialize DSP components
    let mut spectrum_processor = SpectrumProcessor::new();
    let mut demodulator: Box<dyn Demodulator> = create_demodulator(demod_mode, sample_rate as f32);
    let mut lowpass_filter = IirLowPassFilter::new(LOWPASS_CUTOFF_HZ, sample_rate as f32);

    let decimation = (sample_rate / AUDIO_RATE) as usize;
    
    // Calculate the duration of each chunk for real-time pacing
    let samples_per_chunk = (WAV_CHUNK_SIZE / 2) as f64; // I/Q pairs
    let chunk_duration_secs = samples_per_chunk / sample_rate as f64;
    let chunk_duration = std::time::Duration::from_secs_f64(chunk_duration_secs);

    // Skip to start position if resuming
    let mut current_position = 0usize;
    if start_position > 0 {
        // Skip samples to reach the start position
        let samples_to_skip = start_position;
        for _ in reader.samples::<i16>().take(samples_to_skip) {}
        current_position = start_position;
    }

    loop {
        if tx.is_closed() {
            break;
        }

        let loop_start = std::time::Instant::now();

        // Read chunk of samples from WAV file
        let samples: Vec<i16> = reader
            .samples::<i16>()
            .take(WAV_CHUNK_SIZE)
            .filter_map(|s| s.ok())
            .collect();

        if samples.is_empty() {
            // End of file - could loop or stop
            break;
        }

        // Update position and send to app
        current_position += samples.len();
        if tx.blocking_send(Message::WavPosition(current_position)).is_err() {
            break;
        }

        // Convert i16 samples to u8 format (matching RTL-SDR format)
        let buffer: Vec<u8> = samples
            .iter()
            .map(|&s| ((s as i32 + 32768) / 256) as u8)
            .collect();

        // Process FFT for UI (use subset for performance)
        let spectrum_size = buffer.len().min(FFT_SIZE);
        let spectrum = spectrum_processor.process_fft(&buffer[0..spectrum_size]);
        if tx.blocking_send(Message::SpectrumData(spectrum)).is_err() {
            break;
        }

        // Demodulate and create audio
        let audio_samples = demodulate_and_filter(
            &buffer,
            demodulator.as_mut(),
            &mut lowpass_filter,
            decimation,
        );

        // Send audio to output
        if !audio_samples.is_empty() {
            let source = SamplesBuffer::new(1, AUDIO_RATE, audio_samples);
            sink.append(source);
        }

        // Sleep to maintain real-time playback speed
        let elapsed = loop_start.elapsed();
        if let Some(remaining) = chunk_duration.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }

    // Don't wait for audio to finish - let it keep playing in the global sink
    // sink.sleep_until_end();
    
    let _ = tx.blocking_send(Message::Error("WAV file playback ended".to_string()));
    Ok(())
}
