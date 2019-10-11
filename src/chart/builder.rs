use super::context::ChartContext;

use crate::coord::{AsRangedCoord, RangedCoord, Shift};
use crate::drawing::backend::DrawingBackend;
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::style::{IntoTextStyle, SizeDesc, TextStyle};

/// The enum used to specify the position of label area.
/// This is used when we configure the label area size with the API `set_label_area_size`
pub enum LabelAreaPosition {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
}

/// The helper object to create a chart context, which is used for the high-level figure drawing.
/// With the hlep of this object, we can convert a basic drawing area into a chart context, which
/// allows the high-level chartting API beening used on the drawing area.
pub struct ChartBuilder<'a, 'b, DB: DrawingBackend> {
    label_area_size: [u32; 4], // [upper, lower, left, right]
    root_area: &'a DrawingArea<DB, Shift>,
    title: Option<(String, TextStyle<'b>)>,
    margin: [u32; 4],
}

impl<'a, 'b, DB: DrawingBackend> ChartBuilder<'a, 'b, DB> {
    /// Create a chart builder on the given drawing area
    /// - `root`: The root drawing area
    /// - Returns: The chart builder object
    pub fn on(root: &'a DrawingArea<DB, Shift>) -> Self {
        Self {
            label_area_size: [0; 4],
            root_area: root,
            title: None,
            margin: [0; 4],
        }
    }

    /// Set the margin size of the chart (applied for top, bottom, left and right at the same time)
    /// - `size`: The size of the chart margin.
    pub fn margin<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.margin = [size, size, size, size];
        self
    }

    /// Set the top margin of current chart
    /// - `size`: The size of the top margin.
    pub fn margin_top<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.margin[0] = size;
        self
    }

    /// Set the bottom margin of current chart
    /// - `size`: The size of the bottom margin.
    pub fn margin_bottom<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.margin[1] = size;
        self
    }

    /// Set the left margin of current chart
    /// - `size`: The size of the left margin.
    pub fn margin_left<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.margin[2] = size;
        self
    }

    /// Set the right margin of current chart
    /// - `size`: The size of the right margin.
    pub fn margin_right<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.margin[3] = size;
        self
    }

    /// Set the size of X label area
    /// - `size`: The height of the x label area, if x is 0, the chart doesn't have the X label area
    pub fn x_label_area_size<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.label_area_size[1] = size;
        self
    }

    /// Set the size of the Y label area
    /// - `size`: The width of the Y label area. If size is 0, the chart doesn't have Y label area
    pub fn y_label_area_size<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.label_area_size[2] = size;
        self
    }

    /// Set the size of X label area on the top of the chart
    /// - `size`: The height of the x label area, if x is 0, the chart doesn't have the X label area
    pub fn top_x_label_area_size<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.label_area_size[0] = size;
        self
    }

    /// Set the size of the Y label area on the right side
    /// - `size`: The width of the Y label area. If size is 0, the chart doesn't have Y label area
    pub fn right_y_label_area_size<S: SizeDesc>(&mut self, size: S) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.label_area_size[3] = size;
        self
    }

    /// Set a label area size
    /// - `pos`: THe position where the label area locted
    /// - `size`: The size of the label area size
    pub fn set_label_area_size<S: SizeDesc>(
        &mut self,
        pos: LabelAreaPosition,
        size: S,
    ) -> &mut Self {
        let size = size.in_pixels(self.root_area).max(0) as u32;
        self.label_area_size[pos as usize] = size;
        self
    }

    /// Set the caption of the chart
    /// - `caption`: The caption of the chart
    /// - `style`: The text style
    /// - Note: If the caption is set, the margin option will be ignored
    pub fn caption<S: AsRef<str>, Style: IntoTextStyle<'b>>(
        &mut self,
        caption: S,
        style: Style,
    ) -> &mut Self {
        self.title = Some((
            caption.as_ref().to_string(),
            style.into_text_style(self.root_area),
        ));
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
        ChartContext<'a, DB, RangedCoord<X::CoordDescType, Y::CoordDescType>>,
        DrawingAreaErrorKind<DB::ErrorType>,
    > {
        let mut label_areas = [None, None, None, None];

        let mut drawing_area = DrawingArea::clone(self.root_area);

        if *self.margin.iter().max().unwrap_or(&0) > 0 {
            drawing_area = drawing_area.margin(
                self.margin[0] as i32,
                self.margin[1] as i32,
                self.margin[2] as i32,
                self.margin[3] as i32,
            );
        }

        if let Some((ref title, ref style)) = self.title {
            drawing_area = drawing_area.titled(title, style.clone())?;
        }

        let (w, h) = drawing_area.dim_in_pixel();

        let mut actual_drawing_area_pos = [0, h as i32, 0, w as i32];

        for (idx, (dx, dy)) in (0..4).map(|idx| (idx, [(0, -1), (0, 1), (-1, 0), (1, 0)][idx])) {
            let size = self.label_area_size[idx] as i32;

            let split_point = if dx + dy < 0 { size } else { -size };

            actual_drawing_area_pos[idx] += split_point;
        }

        let mut splitted: Vec<_> = drawing_area
            .split_by_breakpoints(
                &actual_drawing_area_pos[2..4],
                &actual_drawing_area_pos[0..2],
            )
            .into_iter()
            .map(Some)
            .collect();

        for (src_idx, dst_idx) in [1, 7, 3, 5].iter().zip(0..4) {
            let (h, w) = splitted[*src_idx].as_ref().unwrap().dim_in_pixel();
            if h > 0 && w > 0 {
                std::mem::swap(&mut label_areas[dst_idx], &mut splitted[*src_idx]);
            }
        }

        std::mem::swap(&mut drawing_area, splitted[4].as_mut().unwrap());

        let mut pixel_range = drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        let mut x_label_area = [None, None];
        let mut y_label_area = [None, None];

        std::mem::swap(&mut x_label_area[0], &mut label_areas[0]);
        std::mem::swap(&mut x_label_area[1], &mut label_areas[1]);
        std::mem::swap(&mut y_label_area[0], &mut label_areas[2]);
        std::mem::swap(&mut y_label_area[1], &mut label_areas[3]);

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    #[test]
    fn test_label_area_size() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartBuilder::on(&drawing_area);

        chart
            .x_label_area_size(10)
            .y_label_area_size(20)
            .top_x_label_area_size(30)
            .right_y_label_area_size(40);
        assert_eq!(chart.label_area_size[1], 10);
        assert_eq!(chart.label_area_size[2], 20);
        assert_eq!(chart.label_area_size[0], 30);
        assert_eq!(chart.label_area_size[3], 40);

        chart.set_label_area_size(LabelAreaPosition::Left, 100);
        chart.set_label_area_size(LabelAreaPosition::Right, 200);
        chart.set_label_area_size(LabelAreaPosition::Top, 300);
        chart.set_label_area_size(LabelAreaPosition::Bottom, 400);

        assert_eq!(chart.label_area_size[0], 300);
        assert_eq!(chart.label_area_size[1], 400);
        assert_eq!(chart.label_area_size[2], 100);
        assert_eq!(chart.label_area_size[3], 200);
    }

    #[test]
    fn test_margin_configure() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartBuilder::on(&drawing_area);

        chart.margin(5);
        assert_eq!(chart.margin[0], 5);
        assert_eq!(chart.margin[1], 5);
        assert_eq!(chart.margin[2], 5);
        assert_eq!(chart.margin[3], 5);

        chart.margin_top(10);
        chart.margin_bottom(11);
        chart.margin_left(12);
        chart.margin_right(13);
        assert_eq!(chart.margin[0], 10);
        assert_eq!(chart.margin[1], 11);
        assert_eq!(chart.margin[2], 12);
        assert_eq!(chart.margin[3], 13);
    }

    #[test]
    fn test_caption() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartBuilder::on(&drawing_area);

        chart.caption("This is a test case", ("Arial", 10));

        assert_eq!(chart.title.as_ref().unwrap().0, "This is a test case");
        assert_eq!(chart.title.as_ref().unwrap().1.font.get_name(), "Arial");
        assert_eq!(chart.title.as_ref().unwrap().1.font.get_size(), 10.0);
        assert_eq!(
            chart.title.as_ref().unwrap().1.color.to_rgba(),
            BLACK.to_rgba()
        );
    }
}
