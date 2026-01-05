use iced::widget::{button, column, text, container, canvas, row, text_input, pick_list};
use iced::{Element, Length, Center, Theme, Color, Point, Rectangle, Subscription, Size};
use iced::mouse;

use crate::dsp::sdr_stream;

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
    current_freq: u64,
    is_connected: bool,
    waterfall: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ConnectToggle,
    FreqInputChanged(String),
    FreqUnitChanged(FreqUnit),
    DemodModeChanged(DemodMode),
    SetFrequency,
    SpectrumData(Vec<u8>),
    Error(String),
}

impl Default for SdrApp {
    fn default() -> Self {
        Self {
            status: "Disconnected".to_string(),
            freq_input: "100".to_string(),
            freq_unit: FreqUnit::MHz,
            demod_mode: DemodMode::FM,
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
        let controls = row![
            text_input("Frequency", &self.freq_input)
                .on_input(Message::FreqInputChanged)
                .on_submit(Message::SetFrequency)
                .padding(10)
                .width(Length::Fixed(150.0)),
            pick_list(
                &FreqUnit::ALL[..],
                Some(self.freq_unit),
                Message::FreqUnitChanged
            ),
            pick_list(
                &DemodMode::ALL[..],
                Some(self.demod_mode),
                Message::DemodModeChanged
            ),
            button("Set Freq").on_press(Message::SetFrequency),
            button(if self.is_connected { "Disconnect" } else { "Connect" })
                .on_press(Message::ConnectToggle),
        ].spacing(10);

        let content = column![
            text("RTL-SDR Controller").size(30),
            text(&self.status).size(16),
            controls,
            canvas(SpectrumProgram { waterfall: &self.waterfall })
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .spacing(20)
        .align_x(Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_connected {
            sdr_stream(self.current_freq, self.demod_mode)
        } else {
            Subscription::none()
        }
    }
}

fn sdr_stream(frequency: u64, demod_mode: DemodMode) -> Subscription<Message> {
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

struct SpectrumProgram<'a> {
    waterfall: &'a Vec<Vec<u8>>,
}

impl<'a> canvas::Program<Message> for SpectrumProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let background = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::BLACK);

        if let Some(latest) = self.waterfall.first() {
             // Draw Spectrum Line
             let width = bounds.width;
             let height = bounds.height / 2.0;
             let len = latest.len() as f32;
             
             let path = canvas::Path::new(|b| {
                 b.move_to(Point::new(0.0, height));
                 for (i, &val) in latest.iter().enumerate() {
                     let x = (i as f32 / len) * width;
                     let y = height - (val as f32 / 255.0) * height;
                     b.line_to(Point::new(x, y));
                 }
                 b.line_to(Point::new(width, height));
             });
             
             frame.stroke(&path, canvas::Stroke::default().with_color(Color::from_rgb(0.0, 1.0, 0.0)).with_width(1.0));
        }

        // Draw Waterfall (simplified)
        let start_y = bounds.height / 2.0;
        
        for (row_idx, row_data) in self.waterfall.iter().enumerate() {
            let y = start_y + row_idx as f32 * 2.0; // 2px per row
            if y > bounds.height { break; }
            
            let len = row_data.len();
            let w_step = bounds.width / len as f32;
            
            // Subsample x4 for speed
            for (col_idx, &val) in row_data.iter().enumerate().step_by(4) { 
                 let x = col_idx as f32 * w_step;
                 let color = Color::from_rgb(val as f32 / 255.0, 0.0, 1.0 - (val as f32 / 255.0));
                 frame.fill_rectangle(Point::new(x, y), Size::new(w_step * 4.0, 2.0), color);
            }
        }

        vec![frame.into_geometry()]
    }
}
