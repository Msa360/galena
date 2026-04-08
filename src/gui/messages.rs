use crate::app::{DemodMode, Source, AudioDevice};

#[derive(Debug, Clone)]
pub enum Message {
    PlayPause,
    DemodModeChanged(DemodMode),
    SourceChanged(Source),
    AudioDeviceChanged(AudioDevice),
    BrowseWavFile,
    FilePathChanged(String),
    FreqIncrement(u64),
    FreqDecrement(u64),
    SpectrumData(Vec<u8>),
    WavPosition(usize),
    SdrConnectionStatus(bool),
    AudioDeviceError(String),
    Error(String),
}