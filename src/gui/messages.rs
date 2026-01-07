use crate::app::{DemodMode, Source};

#[derive(Debug, Clone)]
pub enum Message {
    PlayPause,
    DemodModeChanged(DemodMode),
    SourceChanged(Source),
    BrowseWavFile,
    FilePathChanged(String),
    FreqIncrement(u64),
    FreqDecrement(u64),
    SpectrumData(Vec<u8>),
    WavPosition(usize),
    PauseStream,
    ResumeStream,
    SdrConnectionStatus(bool),
    Error(String),
}