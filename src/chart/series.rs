use super::ChartContext;
use crate::coord::CoordTranslate;
use crate::drawing::backend::{BackendCoord, DrawingErrorKind};
use crate::drawing::{DrawingAreaErrorKind, DrawingBackend};
use crate::element::{EmptyElement, IntoDynElement, MultiLineText, Rectangle};
use crate::style::{IntoFont, ShapeStyle, TextStyle, Transparent};

pub enum SeriesLabelPosition {
    UpperRight,
    MiddleRight,
    LowerRight,
    Coordinate(i32, i32),
}

impl SeriesLabelPosition {
    fn layout_label_area(&self, label_dim: (i32, i32), area_dim: (u32, u32)) -> (i32, i32) {
        match self {
            SeriesLabelPosition::UpperRight => (area_dim.0 as i32 - label_dim.0 as i32, 0),
            SeriesLabelPosition::MiddleRight => (
                area_dim.0 as i32 - label_dim.0 as i32,
                (area_dim.1 as i32 - label_dim.1 as i32) / 2,
            ),
            SeriesLabelPosition::LowerRight => (
                area_dim.0 as i32 - label_dim.0 as i32,
                area_dim.1 as i32 - label_dim.1 as i32,
            ),
            SeriesLabelPosition::Coordinate(x, y) => (*x, *y),
        }
    }
}

/// The struct to sepcify the series label of a target chart context
pub struct SeriesLabelStyle<'a, 'b, DB: DrawingBackend, CT: CoordTranslate> {
    target: &'b mut ChartContext<'a, DB, CT>,
    position: SeriesLabelPosition,
    legend_area_size: u32,
    border_style: ShapeStyle<'b>,
    background: ShapeStyle<'b>,
    label_font: Option<TextStyle<'b>>,
    margin: u32,
}

impl<'a, 'b, DB: DrawingBackend + 'a, CT: CoordTranslate> SeriesLabelStyle<'a, 'b, DB, CT> {
    pub(super) fn new(target: &'b mut ChartContext<'a, DB, CT>) -> Self {
        Self {
            target,
            position: SeriesLabelPosition::MiddleRight,
            legend_area_size: 30,
            border_style: (&Transparent).into(),
            background: (&Transparent).into(),
            label_font: None,
            margin: 10,
        }
    }

    /// Set the series label positioning style
    /// `pos` - The positioning style
    pub fn position(&mut self, pos: SeriesLabelPosition) -> &mut Self {
        self.position = pos;
        self
    }

    pub fn margin(&mut self, value: u32) -> &mut Self {
        self.margin = value;
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
    pub fn border_style<S: Into<ShapeStyle<'b>>>(&mut self, style: S) -> &mut Self {
        self.border_style = style.into();
        self
    }

    /// Set the background style
    /// `style` - The style of the border
    pub fn background_style<S: Into<ShapeStyle<'b>>>(&mut self, style: S) -> &mut Self {
        self.background = style.into();
        self
    }

    /// Set the series label font
    /// `font` - The font
    pub fn label_font<F: Into<TextStyle<'b>>>(&mut self, font: F) -> &mut Self {
        self.label_font = Some(font.into());
        self
    }

    /// Draw the series label area
    pub fn draw(&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        let drawing_area = self.target.plotting_area().strip_coord_spec();
        let default_font = ("Arial", 12).into_font();
        let default_style: TextStyle = (&default_font).into();

        let font = {
            let mut temp = None;
            std::mem::swap(&mut self.label_font, &mut temp);
            temp.unwrap_or(default_style)
        };

        let mut label_element = MultiLineText::<_, &str>::new((0, 0), &font);
        let mut funcs = vec![];

        for anno in self.target.series_anno.iter() {
            let label_text = anno.get_label();
            let draw_func = anno.get_draw_func();

            if label_text == "" && draw_func.is_none() {
                continue;
            }

            funcs.push(
                draw_func.unwrap_or_else(|| &|p: BackendCoord| EmptyElement::at(p).into_dyn()),
            );
            label_element.push_line(label_text);
        }

        let (mut w, mut h) = label_element
            .estimate_dimension()
            .map_err(|e| DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(e)))?;

        let margin = self.margin as i32;

        w += self.legend_area_size as i32 + margin * 2;
        h += margin * 2;

        let (area_w, area_h) = drawing_area.dim_in_pixel();

        let (label_x, label_y) = self.position.layout_label_area((w, h), (area_w, area_h));

        label_element.relocate((
            label_x + self.legend_area_size as i32 + margin,
            label_y + margin,
        ));

        drawing_area.draw(&Rectangle::new(
            [(label_x, label_y), (label_x + w, label_y + h)],
            self.background.filled(),
        ))?;
        drawing_area.draw(&Rectangle::new(
            [(label_x, label_y), (label_x + w, label_y + h)],
            self.border_style.clone(),
        ))?;
        drawing_area.draw(&label_element)?;

        for (((_, y0), (_, y1)), make_elem) in label_element
            .compute_line_layout()
            .map_err(|e| DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(e)))?
            .into_iter()
            .zip(funcs.into_iter())
        {
            let legend_element = make_elem((label_x + margin, (y0 + y1) / 2));
            drawing_area.draw(&legend_element)?;
        }

        Ok(())
    }
}
