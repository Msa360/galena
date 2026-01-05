use crate::app::{DemodMode, FreqUnit, SourceType};

#[derive(Debug, Clone)]
pub enum Message {
    ConnectToggle,
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
    Error(String),
}