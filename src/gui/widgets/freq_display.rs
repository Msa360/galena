use iced::widget::{button, row, text, container, column};
use iced::{Element, Alignment, Border};

/// Create a frequency display with clickable digits
pub fn view<Message: Clone + 'static>(
    frequency: u64,
    on_increment: impl Fn(u64) -> Message + 'static,
    on_decrement: impl Fn(u64) -> Message + 'static,
) -> Element<'static, Message> {
    let freq_str = format!("{frequency:09}"); // Ensure minimum 9 digits
    let num_digits = freq_str.len();
    
    let mut digits_row = row![].spacing(0).align_y(Alignment::Center);
    
    for (i, digit_char) in freq_str.chars().enumerate() {
        let position = num_digits - i - 1;
        let multiplier = 10_u64.pow(position as u32);
        
        // Create a column with two buttons for top/bottom half that show arrows on hover
        let up_btn = button(
            container(text("▲").size(10))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill)
        )
            .on_press(on_increment(multiplier))
            .padding(0)
            .style(move |_theme, status| {
                let (background, text_color) = match status {
                    button::Status::Hovered => (
                        iced::Color::from_rgba(0.5, 0.7, 1.0, 0.3),
                        iced::Color::from_rgb(0.0, 0.0, 0.0),
                    ),
                    button::Status::Pressed => (
                        iced::Color::from_rgba(0.3, 0.5, 0.9, 0.5),
                        iced::Color::from_rgb(0.0, 0.0, 0.0),
                    ),
                    _ => (
                        iced::Color::TRANSPARENT,
                        iced::Color::TRANSPARENT,
                    ),
                };
                button::Style {
                    background: Some(iced::Background::Color(background)),
                    text_color,
                    border: Border::default(),
                    ..Default::default()
                }
            })
            .height(iced::Length::Fixed(12.0))
            .width(iced::Length::Fixed(16.0));
        
        let digit_display = container(text(digit_char.to_string()).size(24))
            .center_x(iced::Length::Fixed(16.0))
            .center_y(iced::Length::Shrink);
        
        let down_btn = button(
            container(text("▼").size(10))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill)
        )
            .on_press(on_decrement(multiplier))
            .padding(0)
            .style(move |_theme, status| {
                let (background, text_color) = match status {
                    button::Status::Hovered => (
                        iced::Color::from_rgba(0.5, 0.7, 1.0, 0.3),
                        iced::Color::from_rgb(0.0, 0.0, 0.0),
                    ),
                    button::Status::Pressed => (
                        iced::Color::from_rgba(0.3, 0.5, 0.9, 0.5),
                        iced::Color::from_rgb(0.0, 0.0, 0.0),
                    ),
                    _ => (
                        iced::Color::TRANSPARENT,
                        iced::Color::TRANSPARENT,
                    ),
                };
                button::Style {
                    background: Some(iced::Background::Color(background)),
                    text_color,
                    border: Border::default(),
                    ..Default::default()
                }
            })
            .height(iced::Length::Fixed(12.0))
            .width(iced::Length::Fixed(16.0));
        
        // Stack them to create a clickable digit
        let digit_col = column![up_btn, digit_display, down_btn]
            .align_x(Alignment::Center)
            .spacing(0);
        
        let digit_container = container(digit_col)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
                border: Border {
                    color: iced::Color::from_rgb(0.7, 0.7, 0.7),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .padding(2);
        
        digits_row = digits_row.push(digit_container);
        
        // Add space separator every 3 digits from the right
        if position > 0 && position % 3 == 0 {
            digits_row = digits_row.push(container(text(" ")).width(iced::Length::Fixed(8.0)));
        }
    }
    
    digits_row = digits_row.push(container(text(" Hz").size(20)).padding(8));
    
    container(digits_row)
        .padding(10)
        .into()
}
