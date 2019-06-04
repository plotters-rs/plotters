use super::ChartContext;
use crate::coord::CoordTranslate;
use crate::drawing::DrawingBackend;
use crate::style::{ShapeStyle, TextStyle, Transparent};

pub enum SeriesLabelPosition {
    UpperRight,
    MiddleRight,
    LowerRight,
    Coordinate(i32, i32),
}

/// The struct to sepcify the series label of a target chart context
pub struct SeriesLabelStyle<'a, DB: DrawingBackend, CT: CoordTranslate> {
    #[allow(dead_code)]
    target: &'a mut ChartContext<DB, CT>,
    position: SeriesLabelPosition,
    legend_area_size: u32,
    border_style: ShapeStyle<'a>,
    background: ShapeStyle<'a>,
    label_font: Option<TextStyle<'a>>,
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate> SeriesLabelStyle<'a, DB, CT> {
    pub(super) fn new(target: &'a mut ChartContext<DB, CT>) -> Self {
        Self {
            target,
            position: SeriesLabelPosition::MiddleRight,
            legend_area_size: 50,
            border_style: (&Transparent).into(),
            background: (&Transparent).into(),
            label_font: None,
        }
    }

    /// Set the series label positioning style
    /// `pos` - The positioning style
    pub fn position(&mut self, pos: SeriesLabelPosition) -> &mut Self {
        self.position = pos;
        self
    }

    /// Set the size of legend area
    /// `size` - The size of legend area in pixel
    pub fn legend_area_size(&mut self, size: u32) -> &mut Self {
        self.legend_area_size = size;
        self
    }

    /// Set the style of the label series area
    /// `style` - The style of the border
    pub fn border_style<S: Into<ShapeStyle<'a>>>(&mut self, style: S) -> &mut Self {
        self.border_style = style.into();
        self
    }

    /// Set the background style
    /// `style` - The style of the border
    pub fn background_style<S: Into<ShapeStyle<'a>>>(&mut self, style: S) -> &mut Self {
        self.background = style.into();
        self
    }

    /// Set the series label font
    /// `font` - The font
    pub fn label_font<F: Into<TextStyle<'a>>>(&mut self, font: F) -> &mut Self {
        self.label_font = Some(font.into());
        self
    }
}
