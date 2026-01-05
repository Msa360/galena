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
    SpectrumData(Vec<u8>),
    Error(String),
}