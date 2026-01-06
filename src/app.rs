use iced::widget::{button, column, text, container, row, pick_list, tooltip};
use iced::{Element, Length, Subscription};

use crate::gui::{Message, stream, widgets::{Waterfall, freq_display}, components::{basic_tooltip}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DemodMode {
    #[default]
    FM,
    Raw,
    USB,
    LSB,
}

impl DemodMode {
    pub const ALL: [DemodMode; 4] = [DemodMode::FM, DemodMode::Raw, DemodMode::USB, DemodMode::LSB];
}

impl std::fmt::Display for DemodMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DemodMode::FM => "FM",
                DemodMode::Raw => "AM",
                DemodMode::USB => "USB",
                DemodMode::LSB => "LSB",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum SourceType {
    #[default]
    SDR,
    WavFile,
}

impl SourceType {
    pub const ALL: [SourceType; 2] = [SourceType::SDR, SourceType::WavFile];
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SourceType::SDR => "RTL-SDR",
                SourceType::WavFile => "WAV File",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FreqUnit {
    Hz,
    #[default]
    MHz,
    GHz,
}

impl FreqUnit {
    pub const ALL: [FreqUnit; 3] = [FreqUnit::Hz, FreqUnit::MHz, FreqUnit::GHz];
}

impl std::fmt::Display for FreqUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FreqUnit::Hz => "Hz",
                FreqUnit::MHz => "MHz",
                FreqUnit::GHz => "GHz",
            }
        )
    }
}

pub struct SdrApp {
    freq_input: String,
    freq_unit: FreqUnit,
    demod_mode: DemodMode,
    source_type: SourceType,
    file_path: String,
    current_freq: u64,
    is_playing: bool,
    sdr_connected: bool,
    waterfall: Vec<Vec<u8>>,
    wav_position: usize,
}

impl Default for SdrApp {
    fn default() -> Self {
        Self {
            freq_input: "100".to_string(),
            freq_unit: FreqUnit::MHz,
            demod_mode: DemodMode::FM,
            source_type: SourceType::SDR,
            file_path: String::new(),
            current_freq: 100_000_000,
            is_playing: false,
            sdr_connected: false,
            waterfall: Vec::new(),
            wav_position: 0,
        }
    }
}

impl SdrApp {
    fn is_source_ready(&self) -> bool {
        match self.source_type {
            SourceType::SDR => self.sdr_connected,
            SourceType::WavFile => !self.file_path.is_empty(),
        }
    }

    fn get_source_ready_message(&self) -> Option<String> {
        if self.is_source_ready() {
            return None;
        }
        
        Some(match self.source_type {
            SourceType::SDR => "Please connect an RTL-SDR device to start playback".to_string(),
            SourceType::WavFile => "Please select a WAV file to start playback".to_string(),
        })
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PlayPause => {
                self.is_playing = !self.is_playing;
                if !self.is_playing {
                    self.sdr_connected = false;
                }
            }
            Message::PauseStream => {
                // This message is sent from the stream when it receives pause signal
            }
            Message::ResumeStream => {
                // This message is sent from the stream when it receives resume signal
            }
            Message::FreqInputChanged(val) => {
                self.freq_input = val;
            }
            Message::FreqUnitChanged(unit) => {
                self.freq_unit = unit;
            }
            Message::DemodModeChanged(mode) => {
                self.demod_mode = mode;
            }
            Message::SourceTypeChanged(source) => {
                self.source_type = source;
            }
            Message::BrowseWavFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("WAV files", &["wav"])
                    .pick_file()
                {
                    self.file_path = path.display().to_string();
                }
            }
            Message::FilePathChanged(path) => {
                self.file_path = path;
            }
            Message::SetFrequency => {
                if let Ok(val) = self.freq_input.parse::<f64>() {
                    let multiplier = match self.freq_unit {
                        FreqUnit::Hz => 1.0,
                        FreqUnit::MHz => 1_000_000.0,
                        FreqUnit::GHz => 1_000_000_000.0,
                    };
                    self.current_freq = (val * multiplier) as u64;
                }
            }
            Message::FreqIncrement(multiplier) => {
                self.current_freq = self.current_freq.saturating_add(multiplier);
                self.freq_input = (self.current_freq as f64 / 1_000_000.0).to_string();
            }
            Message::FreqDecrement(multiplier) => {
                self.current_freq = self.current_freq.saturating_sub(multiplier);
                self.freq_input = (self.current_freq as f64 / 1_000_000.0).to_string();
            }
            Message::SpectrumData(data) => {
                self.waterfall.insert(0, data);
                if self.waterfall.len() > 100 {
                    self.waterfall.pop();
                }
            }
            Message::WavPosition(pos) => {
                self.wav_position = pos;
            }
            Message::SdrConnectionStatus(connected) => {
                self.sdr_connected = connected;
            }
            Message::Error(_e) => {
                self.sdr_connected = false;
                self.is_playing = false;
                self.wav_position = 0;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        // Top bar with source selector in the left
        let source_selector = row![
            text("Source:").size(14),
            pick_list(
                &SourceType::ALL[..],
                Some(self.source_type),
                Message::SourceTypeChanged
            ),
        ].spacing(10);

        let top_bar = container(source_selector)
            .padding(10);

        // Control panel in the center
        let mut control_row = row![].spacing(15);

        // Show frequency controls for SDR
        if self.source_type == SourceType::SDR {
            control_row = control_row.push(
                freq_display::view(
                    self.current_freq,
                    Message::FreqIncrement,
                    Message::FreqDecrement
                )
            );
        } else {
            // Show file browser for WAV file
            control_row = control_row.push(
                button("Browse WAV File...").on_press(Message::BrowseWavFile)
            );
            if !self.file_path.is_empty() {
                let filename = std::path::Path::new(&self.file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&self.file_path);
                control_row = control_row.push(
                    text(filename).size(12)
                );
            }
        }

        control_row = control_row.push(
            pick_list(
                &DemodMode::ALL[..],
                Some(self.demod_mode),
                Message::DemodModeChanged
            )
        );
        let play_button = button(if self.is_playing { "⏸ Pause" } else { "▶ Play" });
        let play_button = if self.is_playing || self.is_source_ready() {
            play_button.on_press(Message::PlayPause)
        } else {
            play_button
        };
        
        // Wrap play button in tooltip if source is not ready
        let play_button_element: Element<Message> = if let Some(message) = self.get_source_ready_message() {
            basic_tooltip(
                play_button,
                message,
                tooltip::Position::Top
            )
        } else {
            play_button.into()
        };
        
        control_row = control_row.push(play_button_element);

        let controls_panel = container(control_row)
            .padding(10)
            .center_x(Length::Fill);

        let _title = "Galena";

        // Connection status indicator (only for SDR mode)
        let mut content_col = column![top_bar];
        
        if self.source_type == SourceType::SDR {
            let indicator = if self.sdr_connected {
                text("● RTL-SDR Connected")
                    .style(|_theme| text::Style { color: Some(iced::Color::from_rgb(0.0, 0.8, 0.0)) })
            } else {
                text("● RTL-SDR Disconnected")
                    .style(|_theme| text::Style { color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0)) })
            };
            content_col = content_col.push(indicator.size(14));
        }
        
        let content = content_col
            .push(controls_panel)
            .push(Waterfall::new(&self.waterfall))
            .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let playing_subscription = if self.is_playing {
            match self.source_type {
                SourceType::SDR => stream::sdr::subscription(self.current_freq, self.demod_mode),
                SourceType::WavFile => stream::wav::subscription(
                    self.file_path.clone(),
                    self.demod_mode,
                    self.wav_position,
                    self.is_playing
                ),
            }
        } else {
            Subscription::none()
        };

        // When in SDR mode and not playing, check connection status
        let connection_check = if self.source_type == SourceType::SDR && !self.is_playing {
            stream::sdr::connection_check_subscription()
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![playing_subscription, connection_check])
    }
}
