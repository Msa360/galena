use iced::widget::{button, column, text, container, row, text_input, pick_list};
use iced::{Element, Length, Subscription};

use crate::gui::{sdr_stream, wav_stream, Waterfall, Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DemodMode {
    #[default]
    FM,
    Raw,
}

impl DemodMode {
    pub const ALL: [DemodMode; 2] = [DemodMode::FM, DemodMode::Raw];
}

impl std::fmt::Display for DemodMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DemodMode::FM => "FM",
                DemodMode::Raw => "Raw (AM)",
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
    status: String,
    freq_input: String,
    freq_unit: FreqUnit,
    demod_mode: DemodMode,
    source_type: SourceType,
    file_path: String,
    current_freq: u64,
    is_connected: bool,
    waterfall: Vec<Vec<u8>>,
}

impl Default for SdrApp {
    fn default() -> Self {
        Self {
            status: "Disconnected".to_string(),
            freq_input: "100".to_string(),
            freq_unit: FreqUnit::MHz,
            demod_mode: DemodMode::FM,
            source_type: SourceType::SDR,
            file_path: String::new(),
            current_freq: 100_000_000,
            is_connected: false,
            waterfall: Vec::new(),
        }
    }
}

impl SdrApp {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ConnectToggle => {
                self.is_connected = !self.is_connected;
                if !self.is_connected {
                    self.status = "Disconnected".to_string();
                }
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
            Message::SpectrumData(data) => {
                self.status = format!("Connected. Freq: {} Hz", self.current_freq);
                self.waterfall.insert(0, data);
                if self.waterfall.len() > 100 {
                    self.waterfall.pop();
                }
            }
            Message::Error(e) => {
                self.status = format!("Error: {}", e);
                self.is_connected = false;
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
                text_input("Frequency", &self.freq_input)
                    .on_input(Message::FreqInputChanged)
                    .on_submit(Message::SetFrequency)
                    .padding(10)
                    .width(Length::Fixed(150.0))
            );
            control_row = control_row.push(
                pick_list(
                    &FreqUnit::ALL[..],
                    Some(self.freq_unit),
                    Message::FreqUnitChanged
                )
            );
            control_row = control_row.push(
                button("Set Freq").on_press(Message::SetFrequency)
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
        control_row = control_row.push(
            button(if self.is_connected { "Disconnect" } else { "Connect" })
                .on_press(Message::ConnectToggle)
        );

        let controls_panel = container(control_row)
            .padding(10)
            .center_x(Length::Fill);

        let title = match self.source_type {
            SourceType::SDR => "RTL-SDR Controller",
            SourceType::WavFile => "WAV File Player",
        };

        let content = column![
            top_bar,
            text(title).size(30),
            text(&self.status).size(16),
            controls_panel,
            Waterfall::new(&self.waterfall)
        ]
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_connected {
            match self.source_type {
                SourceType::SDR => sdr_stream_subscription(self.current_freq, self.demod_mode),
                SourceType::WavFile => wav_stream_subscription(self.file_path.clone(), self.demod_mode),
            }
        } else {
            Subscription::none()
        }
    }
}

fn sdr_stream_subscription(frequency: u64, demod_mode: DemodMode) -> Subscription<Message> {
    use iced::futures::SinkExt;
    
    Subscription::run_with_id(
        (frequency, demod_mode),
        iced::stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            sdr_stream::start_sdr_stream(frequency, demod_mode, tx);
            
            while let Some(msg) = rx.recv().await {
                let _ = output.send(msg).await;
            }
        })
    )
}

fn wav_stream_subscription(file_path: String, demod_mode: DemodMode) -> Subscription<Message> {
    use iced::futures::SinkExt;
    
    Subscription::run_with_id(
        (file_path.clone(), demod_mode),
        iced::stream::channel(100, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            wav_stream::start_wav_stream(file_path, demod_mode, tx);
            
            while let Some(msg) = rx.recv().await {
                let _ = output.send(msg).await;
            }
        })
    )
}
