use super::context::ChartContext;

use crate::coord::{AsRangedCoord, RangedCoord, Shift};
use crate::drawing::backend::DrawingBackend;
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::style::TextStyle;

/// The helper object to create a chart context, which is used for the high-level figure drawing
pub struct ChartBuilder<'a, DB: DrawingBackend> {
    x_label_size: u32,
    y_label_size: u32,
    root_area: &'a DrawingArea<DB, Shift>,
    title: Option<(String, TextStyle<'a>)>,
    margin: u32,
}

impl<'a, DB: DrawingBackend> ChartBuilder<'a, DB> {
    /// Create a chart builder on the given drawing area
    /// - `root`: The root drawing area
    /// - Returns: The chart builder object
    pub fn on(root: &'a DrawingArea<DB, Shift>) -> Self {
        Self {
            x_label_size: 0,
            y_label_size: 0,
            root_area: root,
            title: None,
            margin: 0,
        }
    }

    /// Set the margin size of the chart
    /// - `size`: The size of the chart margin. If the chart builder is titled, we don't apply any
    /// margin
    pub fn margin(&mut self, size: u32) -> &mut Self {
        self.margin = size;
        self
    }

    /// Set the size of X label area
    /// - `size`: The height of the x label area, if x is 0, the chart doesn't have the X label area
    pub fn x_label_area_size(&mut self, size: u32) -> &mut Self {
        self.x_label_size = size;
        self
    }

    /// Set the size of the Y label area
    /// - `size`: The width of the Y label area. If size is 0, the chart doesn't have Y label area
    pub fn y_label_area_size(&mut self, size: u32) -> &mut Self {
        self.y_label_size = size;
        self
    }

    /// Set the caption of the chart
    /// - `caption`: The caption of the chart
    /// - `style`: The text style
    /// - Note: If the caption is set, the margin option will be ignored
    pub fn caption<S: AsRef<str>, Style: Into<TextStyle<'a>>>(
        &mut self,
        caption: S,
        style: Style,
    ) -> &mut Self {
        self.title = Some((caption.as_ref().to_string(), style.into()));
        self
    }

    /// Build the chart with a 2D Cartesian coordinate system. The function will returns a chart
    /// context, where data series can be rendered on.
    /// - `x_spec`: The specification of X axis
    /// - `y_spec`: The specification of Y axis
    /// - Returns: A chart context
    #[allow(clippy::type_complexity)]
    pub fn build_ranged<X: AsRangedCoord, Y: AsRangedCoord>(
        &mut self,
        x_spec: X,
        y_spec: Y,
    ) -> Result<
        ChartContext<DB, RangedCoord<X::CoordDescType, Y::CoordDescType>>,
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let mut x_label_area = None;
        let mut y_label_area = None;

        let mut drawing_area = DrawingArea::clone(self.root_area);

        if self.margin > 0 {
            let s = self.margin as i32;
            drawing_area = drawing_area.margin(s, s, s, s);
        }

        if let Some((ref title, ref style)) = self.title {
            drawing_area = drawing_area.titled(title, style.clone())?;
        }

        if self.x_label_size > 0 {
            let (_, h) = drawing_area.dim_in_pixel();
            let (upper, bottom) =
                drawing_area.split_vertically(h as i32 - self.x_label_size as i32);
            drawing_area = upper;
            x_label_area = Some(bottom);
        }

        if self.y_label_size > 0 {
            let (left, right) = drawing_area.split_horizentally(self.y_label_size as i32);
            drawing_area = right;
            y_label_area = Some(left);

            if let Some(xl) = x_label_area {
                let (_, right) = xl.split_horizentally(self.y_label_size as i32);
                x_label_area = Some(right);
            }
        }

        let mut pixel_range = drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        Ok(ChartContext {
            x_label_area,
            y_label_area,
            drawing_area: drawing_area.apply_coord_spec(RangedCoord::new(
                x_spec,
                y_spec,
                pixel_range,
            )),
            series_anno: vec![],
        })
    }
}
