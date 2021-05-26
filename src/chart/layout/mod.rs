use crate::coord::Shift;

use crate::drawing::DrawingArea;
use crate::style::IntoFont;
use crate::style::{colors, TextStyle};

use plotters_backend::DrawingBackend;

mod nodes;
use nodes::*;

/// The helper object to create a chart context, which is used for the high-level figure drawing.
/// With the help of this object, we can convert a basic drawing area into a chart context, which
/// allows the high-level charting API being used on the drawing area.
pub struct ChartLayout<'a, 'b, DB: DrawingBackend> {
    root_area: &'a DrawingArea<DB, Shift>,
    title_text: Option<String>,
    title_style: TextStyle<'b>,
    nodes: ChartLayoutNodes,
}

impl<'a, 'b, DB: DrawingBackend> ChartLayout<'a, 'b, DB> {
    /// Create a chart builder on the given drawing area
    /// - `root`: The root drawing area
    /// - Returns: The chart layout object
    pub fn new(root: &'a DrawingArea<DB, Shift>) -> Self {
        Self {
            root_area: root,
            title_text: None,
            title_style: TextStyle::from(("serif", 40.0).into_font()),
            nodes: ChartLayoutNodes::new().unwrap(),
        }
    }

    pub fn draw(&mut self) -> Result<&mut Self, Box<dyn std::error::Error + 'a>> {
        let (x_range, y_range) = self.root_area.get_pixel_range();
        let (x_top, y_top) = (x_range.start, y_range.start);
        let (w, h) = (x_range.end - x_top, y_range.end - y_top);
        self.nodes.layout(w as u32, h as u32)?;

        if let Some(title) = self.title_text.as_ref() {
            let extents = self.nodes.get_chart_title_extents().unwrap();
            self.root_area.draw_text(
                title,
                &self.title_style,
                (extents.0 + x_top, extents.1 + y_top),
            )?;
        }

        Ok(self)
    }

    /// Set the chart's title text
    pub fn set_title_text<S: AsRef<str>>(
        &mut self,
        title: S,
    ) -> Result<&mut Self, Box<dyn std::error::Error + 'a>> {
        self.title_text = Some(title.as_ref().to_string());
        let (w, h) = self
            .root_area
            .estimate_text_size(&self.title_text.as_ref().unwrap(), &self.title_style)?;
        self.nodes.set_chart_title_size(w as i32, h as i32)?;
        Ok(self)
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    #[test]
    fn test_label_area_size() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);
    }

    #[test]
    fn test_margin_configure() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);
    }

    #[test]
    fn test_caption() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);
    }
}
*/