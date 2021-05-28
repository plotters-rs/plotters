use crate::{coord::Shift, style::IntoTextStyle};
use paste::paste;

use crate::drawing::{DrawingArea, DrawingAreaErrorKind, LayoutError};
use crate::style::{FontTransform, IntoFont, TextStyle};

use plotters_backend::DrawingBackend;

mod nodes;
pub use nodes::Margin;
use nodes::*;

/// Create the `get_<elm name>_extent` and `get_<elm name>_size` functions
macro_rules! impl_get_extent {
    ($name:ident) => {
        paste! {
            #[doc = "Get the extent (the bounding box) of the `" $name "` container."]
            pub fn [<get_ $name _extent>](
                &self,
            ) -> Result<Extent<i32>, DrawingAreaErrorKind<DB::ErrorType>> {
                let extent = self
                    .nodes
                    .[<get_ $name _extent>]()
                    .ok_or_else(|| LayoutError::ExtentsError)?;

                Ok(extent)
            }
            #[doc = "Get the size of the `" $name "` container."]
            #[doc = "  * **Returns**: An option containing a tuple `(width, height)`."]
            pub fn [<get_ $name _size>](&self) -> Option<(i32, i32)> {
                self.nodes.[<get_ $name _size>]()
            }
        }
    };
}

/// Create all the getters and setters associated with a label that is horizontally layed out
macro_rules! impl_label_horiz {
    ($name:ident) => {
        paste! {
            #[doc = "Recomputes and sets the size of the `" $name "` container."]
            #[doc = "To be called whenever the `" $name "` text/style changes."]
            fn [<recompute_ $name _size>](&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
                let (w, h) = match &self.$name.text.as_ref() {
                    Some(text) => self
                        .root_area
                        .estimate_text_size(text, &self.$name.style)?,
                    None => (0, 0),
                };
                self.nodes.[<set_ $name _size>](w, h)?;

                Ok(())
            }
        }
        impl_label!($name);
    }
}
/// Create all the getters and setters associated with a label that is horizontally layed out
macro_rules! impl_label_vert {
    ($name:ident) => {
        paste! {
            #[doc = "Recomputes and sets the size of the `" $name "` container."]
            #[doc = "To be called whenever the `" $name "` text/style changes."]
            fn [<recompute_ $name _size>](&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
                let (w, h) = match &self.$name.text.as_ref() {
                    Some(text) => self
                        .root_area
                        .estimate_text_size(text, &self.$name.style)?,
                    None => (0, 0),
                };
                // Because this is a label in a vertically layed out label, we swap the width and height
                self.nodes.[<set_ $name _size>](h, w)?;

                Ok(())
            }
        }
        impl_label!($name);
    }
}

/// Create all the getters and setters associated with a labe that don't depend on the label's layout direction
macro_rules! impl_label {
    ($name:ident) => {
        paste! {
            #[doc = "Set the text content of `" $name "`. If `text` is the empty string,"]
            #[doc = "the label will be cleared."]
            pub fn [<set_ $name _text>]<S: AsRef<str>>(
                &mut self,
                text: S,
            ) -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>> {
                Self::set_text(&mut self.$name, text);
                self.[<recompute_ $name _size>]()?;
                Ok(self)
            }
            #[doc = "Clears the text content of the `" $name "` label."]
            pub fn [<clear_ $name _text>](
                &mut self,
            ) -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>> {
                self.[<set_ $name _text>]("")?;
                Ok(self)
            }
            #[doc = "Set the style of the `" $name "` label's text."]
            pub fn [<set_ $name _style>]<Style: IntoTextStyle<'b>>(
                &mut self,
                style: Style,
            ) -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>> {
                Self::set_style(self.root_area, &mut self.$name, style);
                self.[<recompute_ $name _size>]()?;
                Ok(self)
            }
            #[doc = "Set the margin of the `" $name "` container. If `margin` is a single"]
            #[doc = "number, that number is used for all margins. If `margin` is a tuple `(vert,horiz)`,"]
            #[doc = "then the top and bottom margins will be `vert` and the left and right margins will"]
            #[doc = "be `horiz`. To set each margin separately, use a [`Margin`] struct or a four-tuple."]
            pub fn [<set_ $name _margin>]<M: Into<Margin<f32>>>(
                &mut self,
                margin: M,
            ) -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>> {
                self.nodes.[<set_ $name _margin>](margin)?;
                Ok(self)
            }
            #[doc = "Gets the margin of the `" $name "` container."]
            pub fn [<get_ $name _margin>](
                self,
            ) -> Result<Margin<f32>, DrawingAreaErrorKind<DB::ErrorType>> {
                Ok(self.nodes.[<get_ $name _margin>]()?)
            }


        }
    };
}

/// Hold the text and the style of a chart label
struct Label<'a> {
    text: Option<String>,
    style: TextStyle<'a>,
}

/// The helper object to create a chart context, which is used for the high-level figure drawing.
/// With the help of this object, we can convert a basic drawing area into a chart context, which
/// allows the high-level charting API being used on the drawing area.
pub struct ChartLayout<'a, 'b, DB: DrawingBackend> {
    root_area: &'a DrawingArea<DB, Shift>,
    chart_title: Label<'b>,
    top_label: Label<'b>,
    bottom_label: Label<'b>,
    left_label: Label<'b>,
    right_label: Label<'b>,
    nodes: ChartLayoutNodes,
}

impl<'a, 'b, DB: DrawingBackend> ChartLayout<'a, 'b, DB> {
    /// Create a chart builder on the given drawing area
    /// - `root`: The root drawing area
    /// - Returns: The chart layout object
    pub fn new(root: &'a DrawingArea<DB, Shift>) -> Self {
        Self {
            root_area: root,
            chart_title: Label {
                text: None,
                style: TextStyle::from(("serif", 40.0).into_font()),
            },
            top_label: Label {
                text: None,
                style: TextStyle::from(("serif", 25.0).into_font()),
            },
            bottom_label: Label {
                text: None,
                style: TextStyle::from(("serif", 25.0).into_font()),
            },
            left_label: Label {
                text: None,
                style: TextStyle::from(("serif", 25.0).into_font()),
            },
            right_label: Label {
                text: None,
                style: TextStyle::from(("serif", 25.0).into_font()),
            },
            nodes: ChartLayoutNodes::new().unwrap(),
        }
    }

    pub fn draw(&mut self) -> Result<&mut Self, DrawingAreaErrorKind<DB::ErrorType>> {
        let (x_range, y_range) = self.root_area.get_pixel_range();
        let (x_top, y_top) = (x_range.start, y_range.start);
        let (w, h) = (x_range.end - x_top, y_range.end - y_top);
        self.nodes.layout(w as u32, h as u32)?;

        // Draw the horizontally oriented labels
        if let Some(text) = self.chart_title.text.as_ref() {
            let extent = self
                .nodes
                .get_chart_title_extent()
                .ok_or_else(|| LayoutError::ExtentsError)?;
            self.root_area
                .draw_text(text, &self.chart_title.style, (extent.x0, extent.y0))?;
        }

        if let Some(text) = self.top_label.text.as_ref() {
            let extent = self
                .nodes
                .get_top_label_extent()
                .ok_or_else(|| LayoutError::ExtentsError)?;
            self.root_area
                .draw_text(text, &self.top_label.style, (extent.x0, extent.y0))?;
        }
        if let Some(text) = self.bottom_label.text.as_ref() {
            let extent = self
                .nodes
                .get_bottom_label_extent()
                .ok_or_else(|| LayoutError::ExtentsError)?;
            self.root_area
                .draw_text(text, &self.bottom_label.style, (extent.x0, extent.y0))?;
        }
        // Draw the vertically oriented labels
        if let Some(text) = self.left_label.text.as_ref() {
            let extent = self
                .nodes
                .get_left_label_extent()
                .ok_or_else(|| LayoutError::ExtentsError)?;
            self.root_area.draw_text(
                text,
                &self.left_label.style.transform(FontTransform::Rotate270),
                (extent.x0, extent.y1),
            )?;
        }
        if let Some(text) = self.right_label.text.as_ref() {
            let extent = self
                .nodes
                .get_right_label_extent()
                .ok_or_else(|| LayoutError::ExtentsError)?;
            self.root_area.draw_text(
                text,
                &self.right_label.style.transform(FontTransform::Rotate270),
                (extent.x0, extent.y1),
            )?;
        }

        Ok(self)
    }

    /// Return a drawing area which corresponds to the `chart_area` of the current layout.
    /// [`layout`] should be called before this function.
    pub fn get_chart_drawing_area(
        &mut self,
    ) -> Result<DrawingArea<DB, Shift>, DrawingAreaErrorKind<DB::ErrorType>> {
        let chart_area_extent = self.get_chart_area_extent()?;
        Ok(DrawingArea::clone(self.root_area).shrink(
            (chart_area_extent.x0, chart_area_extent.y0),
            chart_area_extent.size(),
        ))
    }

    /// Set the text of a label. If the text is a blank screen, the label is cleared.
    #[inline(always)]
    fn set_text<S: AsRef<str>>(elm: &mut Label, text: S) {
        let text = text.as_ref().to_string();
        elm.text = match text.is_empty() {
            false => Some(text),
            true => None,
        };
    }
    /// Set the style of a label.
    #[inline(always)]
    fn set_style<Style: IntoTextStyle<'b>>(
        root: &'a DrawingArea<DB, Shift>,
        elm: &mut Label<'b>,
        style: Style,
    ) {
        elm.style = style.into_text_style(root);
    }

    impl_get_extent!(top_label);
    impl_get_extent!(bottom_label);
    impl_get_extent!(left_label);
    impl_get_extent!(right_label);
    impl_get_extent!(top_tick_label);
    impl_get_extent!(bottom_tick_label);
    impl_get_extent!(left_tick_label);
    impl_get_extent!(right_tick_label);
    impl_get_extent!(chart_area);
    impl_get_extent!(chart_title);

    impl_label_horiz!(chart_title);
    impl_label_horiz!(bottom_label);
    impl_label_horiz!(top_label);
    impl_label_vert!(left_label);
    impl_label_vert!(right_label);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    fn extent_has_size<T: PartialOrd + Copy + std::ops::Sub + std::ops::Add>(
        extent: Extent<T>,
    ) -> bool {
        (extent.x1 > extent.x0) && (extent.y1 > extent.y0)
    }

    #[test]
    fn test_drawing_of_unset_and_set_chart_title() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);
        chart.draw().unwrap();
        chart.set_chart_title_text("title").unwrap();
        chart.draw().unwrap();

        // Since we set actual text, the extent should have some area
        let extent = chart.get_chart_title_extent().unwrap();
        assert!(extent_has_size(extent));

        // Without any text, the extent shouldn't have any area.
        chart.clear_chart_title_text().unwrap();
        chart.draw().unwrap();
        let extent = chart.get_chart_title_extent().unwrap();
        assert!(!extent_has_size(extent));
    }

    #[test]
    fn test_drawing_of_unset_and_set_labels() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);

        // top_label
        chart.draw().unwrap();
        chart.set_top_label_text("title").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_top_label_extent().unwrap();
        assert!(extent_has_size(extent));

        chart.clear_top_label_text().unwrap();
        chart.draw().unwrap();
        let extent = chart.get_top_label_extent().unwrap();
        assert!(!extent_has_size(extent));

        // bottom_label
        chart.draw().unwrap();
        chart.set_bottom_label_text("title").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_bottom_label_extent().unwrap();
        assert!(extent_has_size(extent));

        chart.clear_bottom_label_text().unwrap();
        chart.draw().unwrap();
        let extent = chart.get_bottom_label_extent().unwrap();
        assert!(!extent_has_size(extent));

        // left_label
        chart.draw().unwrap();
        chart.set_left_label_text("title").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_left_label_extent().unwrap();
        assert!(extent_has_size(extent));

        chart.clear_left_label_text().unwrap();
        chart.draw().unwrap();
        let extent = chart.get_left_label_extent().unwrap();
        assert!(!extent_has_size(extent));

        // right_label
        chart.draw().unwrap();
        chart.set_right_label_text("title").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_right_label_extent().unwrap();
        assert!(extent_has_size(extent));

        chart.clear_right_label_text().unwrap();
        chart.draw().unwrap();
        let extent = chart.get_right_label_extent().unwrap();
        assert!(!extent_has_size(extent));
    }

    #[test]
    fn test_layout_of_horizontal_and_vertical_labels() {
        let drawing_area = create_mocked_drawing_area(800, 600, |_| {});
        let mut chart = ChartLayout::new(&drawing_area);

        // top_label is horizontal
        chart.set_top_label_text("some really long text").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_top_label_extent().unwrap();
        let size = extent.size();
        assert!(size.0 > size.1);
        // left_label is vertically
        chart.set_left_label_text("some really long text").unwrap();
        chart.draw().unwrap();

        let extent = chart.get_left_label_extent().unwrap();
        let size = extent.size();
        assert!(size.1 > size.0);
    }
}
