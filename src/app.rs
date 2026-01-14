use iced::widget::{button, column, text, container, row, pick_list, tooltip};
use iced::{Element, Length, Subscription};
use cpal::traits::{DeviceTrait, HostTrait};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    SdrDevice { index: usize, name: String },
    WavFile,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::SdrDevice { name, .. } => write!(f, "{name}"),
            Source::WavFile => write!(f, "WAV File"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioDevice {
    Default,
    Named {
        id: String,    // Serialized DeviceId for stable identification
        name: String,  // Human-readable name for UI display
    }
}

impl std::fmt::Display for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioDevice::Default => write!(f, "Default Audio Device"),
            AudioDevice::Named { name, .. } => write!(f, "{name}"),
        }
    }
}

/// Enumerate all available audio output devices
fn enumerate_audio_devices() -> Vec<AudioDevice> {
    let mut devices = vec![AudioDevice::Default];

    let host = cpal::default_host();
    match host.output_devices() {
        Ok(output_devices) => {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    devices.push(AudioDevice::Named {
                        id: name.clone(),  // Use device name as stable ID
                        name,
                    });
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to enumerate audio devices: {e:?}");
        }
    }

    devices
}

/// Enumerate all available RTL-SDR devices
fn enumerate_sdr_devices() -> Vec<Source> {
    let mut devices = Vec::new();
    
    // Try to enumerate devices (typically up to 32)
    for index in 0..32 {
        if let Ok(device) = rtl_sdr_rs::RtlSdr::open(rtl_sdr_rs::DeviceId::Index(index)) {
            let name = format!("RTL-SDR #{index}");
            devices.push(Source::SdrDevice { index, name });
            drop(device); // Close the device immediately
        } else {
            // No more devices found
            break;
        }
    }
    
    devices
}

pub struct SdrApp {
    demod_mode: DemodMode,
    available_sources: Vec<Source>,
    selected_source: Option<Source>,
    available_audio_devices: Vec<AudioDevice>,
    selected_audio_device: Option<AudioDevice>,
    file_path: String,
    current_freq: u64,
    is_playing: bool,
    sdr_connected: bool,
    waterfall: Vec<Vec<u8>>,
    wav_position: usize,
}

impl Default for SdrApp {
    fn default() -> Self {
        let sdr_devices = enumerate_sdr_devices();

        // Select first SDR device if available, otherwise None
        let selected_source = sdr_devices.first().cloned();

        // Add WAV file option to available sources
        let mut available_sources = sdr_devices;
        available_sources.push(Source::WavFile);

        // Enumerate audio output devices
        let available_audio_devices = enumerate_audio_devices();
        let selected_audio_device = Some(AudioDevice::Default);

        Self {
            demod_mode: DemodMode::FM,
            available_sources,
            selected_source,
            available_audio_devices,
            selected_audio_device,
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
    fn has_sdr_devices(&self) -> bool {
        self.available_sources.iter().any(|s| matches!(s, Source::SdrDevice { .. }))
    }
    
    fn is_source_ready(&self) -> bool {
        match &self.selected_source {
            Some(Source::SdrDevice { .. }) => self.sdr_connected,
            Some(Source::WavFile) => !self.file_path.is_empty(),
            None => false,
        }
    }

    fn get_source_ready_message(&self) -> Option<String> {
        if self.is_source_ready() {
            return None;
        }
        
        Some(match &self.selected_source {
            Some(Source::SdrDevice { .. }) => "Please connect an RTL-SDR device to start playback".to_string(),
            Some(Source::WavFile) => "Please select a WAV file to start playback".to_string(),
            None => "Please select a source".to_string(),
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
            Message::DemodModeChanged(mode) => {
                self.demod_mode = mode;
            }
            Message::SourceChanged(source) => {
                self.selected_source = Some(source);
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
            Message::FreqIncrement(multiplier) => {
                self.current_freq = self.current_freq.saturating_add(multiplier);
            }
            Message::FreqDecrement(multiplier) => {
                self.current_freq = self.current_freq.saturating_sub(multiplier);
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
            Message::AudioDeviceChanged(device) => {
                // Validate device still exists before selecting
                if self.available_audio_devices.contains(&device) {
                    self.selected_audio_device = Some(device);
                } else {
                    eprintln!("Attempted to select unavailable audio device");
                    self.selected_audio_device = Some(AudioDevice::Default);
                }
            }
            Message::AudioDeviceError(_e) => {
                // Error message from stream when device becomes invalid
                // Device reconstruction will handle fallback in the streaming thread
            }
            Message::Error(_e) => {
                self.sdr_connected = false;
                self.is_playing = false;
                self.wav_position = 0;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        // Top bar with source and audio device selectors
        let source_selector = row![
            text("Source:").size(14),
            pick_list(
                &self.available_sources[..],
                self.selected_source.clone(),
                Message::SourceChanged
            ),
        ].spacing(10);

        // Audio output device selector
        let audio_device_selector = row![
            text("Output:").size(14),
            pick_list(
                &self.available_audio_devices[..],
                self.selected_audio_device.clone(),
                Message::AudioDeviceChanged
            ),
        ].spacing(10);

        let top_bar = container(
            row![source_selector, audio_device_selector].spacing(30)
        )
            .padding(10);

        // Control panel in the center
        let mut control_row = row![].spacing(15);

        // Show frequency controls for SDR
        if matches!(self.selected_source, Some(Source::SdrDevice { .. })) {
            control_row = control_row.push(
                freq_display::view(
                    self.current_freq,
                    Message::FreqIncrement,
                    Message::FreqDecrement
                )
            );
        } else if matches!(self.selected_source, Some(Source::WavFile)) {
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
        
        // Show message if no SDR devices detected
        if !self.has_sdr_devices() {
            let warning = text("⚠ No RTL-SDR devices detected. Please connect an RTL-SDR device.")
                .style(|_theme| text::Style { color: Some(iced::Color::from_rgb(0.9, 0.6, 0.0)) })
                .size(14);
            content_col = content_col.push(warning);
        } else if matches!(self.selected_source, Some(Source::SdrDevice { .. })) {
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
            match &self.selected_source {
                Some(Source::SdrDevice { index, .. }) => {
                    stream::sdr::subscription(
                        self.current_freq,
                        self.demod_mode,
                        *index,
                        self.selected_audio_device.clone()
                    )
                }
                Some(Source::WavFile) => stream::wav::subscription(
                    self.file_path.clone(),
                    self.demod_mode,
                    self.wav_position,
                    self.is_playing,
                    self.selected_audio_device.clone()
                ),
                None => Subscription::none(),
            }
        } else {
            Subscription::none()
        };

        // When in SDR mode and not playing, check connection status
        let connection_check = if matches!(self.selected_source, Some(Source::SdrDevice { .. })) && !self.is_playing {
            if let Some(Source::SdrDevice { index, .. }) = &self.selected_source {
                stream::sdr::connection_check_subscription(*index)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![playing_subscription, connection_check])
    }
}
