use iced::widget::{container, text, tooltip};
use iced::{Element, Color};

/// Creates a styled tooltip with a light grey background
pub fn basic_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    tooltip_text: impl Into<String>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(tooltip_text.into()).size(12))
            .padding(5)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.85, 0.85, 0.85).into()),
                ..Default::default()
            }),
        position,
    )
    .into()
}