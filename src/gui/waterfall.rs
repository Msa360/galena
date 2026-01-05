use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size, Theme};
use iced::mouse;

pub struct Waterfall<'a, Message> {
    waterfall_data: &'a Vec<Vec<u8>>,
    _phantom: std::marker::PhantomData<Message>,
}

impl<'a, Message> Waterfall<'a, Message> {
    pub fn new(waterfall_data: &'a Vec<Vec<u8>>) -> Self {
        Self {
            waterfall_data,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, Message> From<Waterfall<'a, Message>> for iced::Element<'a, Message>
where
    Message: 'a,
{
    fn from(waterfall: Waterfall<'a, Message>) -> Self {
        canvas(WaterfallProgram {
            waterfall: waterfall.waterfall_data,
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}

struct WaterfallProgram<'a> {
    waterfall: &'a Vec<Vec<u8>>,
}

impl<'a, Message> canvas::Program<Message> for WaterfallProgram<'a> {
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
