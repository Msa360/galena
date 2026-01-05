/// WAV file streaming and processing module

use crate::dsp::{AmDemodulator, FmDemodulator, IirLowPassFilter, SpectrumProcessor, process_iq_to_complex};
use crate::app::DemodMode;
use crate::gui::Message;
use rodio::{OutputStreamBuilder, Sink, buffer::SamplesBuffer};
use tokio::sync::mpsc;
use std::path::Path;
use iced::Subscription;

const AUDIO_RATE: u32 = 48_000;
const FFT_SIZE: usize = 16384;

/// Create a subscription for WAV file streaming
pub fn subscription(file_path: String, demod_mode: DemodMode) -> Subscription<Message> {
    use iced::futures::SinkExt;
    
    Subscription::run_with_id(
        (file_path.clone(), demod_mode),
        iced::stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            start_wav_stream(file_path, demod_mode, tx);
            
            while let Some(msg) = rx.recv().await {
                let _ = output.send(msg).await;
            }
        })
    )
}

/// Start WAV file streaming and audio playback
fn start_wav_stream(file_path: String, demod_mode: DemodMode, tx: mpsc::Sender<Message>) {
    std::thread::spawn(move || {
        if let Err(e) = run_wav_loop(&file_path, demod_mode, tx) {
            eprintln!("WAV thread error: {:?}", e);
        }
    });
}

fn run_wav_loop(
    file_path: &str,
    demod_mode: DemodMode,
    tx: mpsc::Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open WAV file
    let mut reader = hound::WavReader::open(Path::new(file_path))
        .map_err(|e| format!("Failed to open WAV file: {:?}", e))?;
    
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    
    // Validate WAV format (expecting I/Q data as interleaved samples)
    if spec.channels != 2 {
        return Err("WAV file must have 2 channels (I/Q data)".into());
    }

    // Initialize audio output
    let stream = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(stream.mixer());
    sink.play();
    
    // Keep stream alive by holding onto it
    let _stream_handle = stream;

    // Initialize DSP components
    let mut spectrum_processor = SpectrumProcessor::new();
    let mut fm_demod = FmDemodulator::new(sample_rate as f32).with_gain(0.8);
    let am_demod = AmDemodulator::new(sample_rate as f32);
    let mut lowpass_filter = IirLowPassFilter::new(10_000.0, sample_rate as f32);
    
    let decimation = (sample_rate / AUDIO_RATE) as usize;
    let chunk_size = 262144; // Process in chunks
    
    // Calculate the duration of each chunk for real-time pacing
    let samples_per_chunk = (chunk_size / 2) as f64; // I/Q pairs
    let chunk_duration_secs = samples_per_chunk / sample_rate as f64;
    let chunk_duration = std::time::Duration::from_secs_f64(chunk_duration_secs);

    loop {
        if tx.is_closed() {
            break;
        }

        let loop_start = std::time::Instant::now();

        // Read chunk of samples from WAV file
        let samples: Vec<i16> = reader
            .samples::<i16>()
            .take(chunk_size)
            .filter_map(|s| s.ok())
            .collect();

        if samples.is_empty() {
            // End of file - could loop or stop
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
            demod_mode,
            &mut fm_demod,
            &am_demod,
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

    // Wait for all buffered audio to finish playing before dropping the sink/stream
    sink.sleep_until_end();
    
    let _ = tx.blocking_send(Message::Error("WAV file playback ended".to_string()));
    Ok(())
}

fn demodulate_and_filter(
    buffer: &[u8],
    demod_mode: DemodMode,
    fm_demod: &mut FmDemodulator,
    am_demod: &AmDemodulator,
    lowpass_filter: &mut IirLowPassFilter,
    decimation: usize,
) -> Vec<f32> {
    let mut audio_samples = Vec::with_capacity(buffer.len() / 2 / decimation);

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
        if i % decimation == 0 {
            audio_samples.push(filtered);
        }
    }

    audio_samples
}
