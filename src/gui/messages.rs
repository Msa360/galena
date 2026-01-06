use crate::app::{DemodMode, FreqUnit, SourceType};

#[derive(Debug, Clone)]
pub enum Message {
    PlayPause,
    FreqInputChanged(String),
    FreqUnitChanged(FreqUnit),
    DemodModeChanged(DemodMode),
    SourceTypeChanged(SourceType),
    BrowseWavFile,
    FilePathChanged(String),
    SetFrequency,
    FreqIncrement(u64),
    FreqDecrement(u64),
    SpectrumData(Vec<u8>),
    WavPosition(usize),
    PauseStream,
    ResumeStream,
    SdrConnectionStatus(bool),
    Error(String),
}