use std::borrow::Borrow;

use plotters_backend::{DrawingBackend, BackendCoord};

use crate::coord::{CoordTranslate, ReverseCoordTranslate, Shift};
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::element::{CoordMapper, PointCollection, Drawable};
use crate::chart::{SeriesAnno, SeriesLabelStyle};



/// The context of the chart. This is the core object of Plotters.
/// Any plot/chart is abstracted as this type, and any data series can be placed to the chart
/// context.
///
/// - To draw a series on a chart context, use [ChartContext::draw_series](struct.ChartContext.html#method.draw_series)
/// - To draw a single element to the chart, you may want to use [ChartContext::plotting_area](struct.ChartContext.html#method.plotting_area)
///
pub struct ChartContext<'a, DB: DrawingBackend, CT: CoordTranslate> {
    pub(crate) x_label_area: [Option<DrawingArea<DB, Shift>>; 2],
    pub(crate) y_label_area: [Option<DrawingArea<DB, Shift>>; 2],
    pub(crate) drawing_area: DrawingArea<DB, CT>,
    pub(crate) series_anno: Vec<SeriesAnno<'a, DB>>,
    pub(crate) drawing_area_pos: (i32, i32),
}


impl<'a, DB: DrawingBackend, CT: ReverseCoordTranslate> ChartContext<'a, DB, CT> {
    /// Convert the chart context into an closure that can be used for coordinate translation
    pub fn into_coord_trans(self) -> impl Fn(BackendCoord) -> Option<CT::From> {
        let coord_spec = self.drawing_area.into_coord_spec();
        move |coord| coord_spec.reverse_translate(coord)
    }
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate> ChartContext<'a, DB, CT> {
    /// Configure the styles for drawing series labels in the chart
    pub fn configure_series_labels<'b>(&'b mut self) -> SeriesLabelStyle<'a, 'b, DB, CT>
    where
        DB: 'a,
    {
        SeriesLabelStyle::new(self)
    }

    /// Get a reference of underlying plotting area
    pub fn plotting_area(&self) -> &DrawingArea<DB, CT> {
        &self.drawing_area
    }

    /// Cast the reference to a chart context to a reference to underlying coordinate specification.
    pub fn as_coord_spec(&self) -> &CT {
        self.drawing_area.as_coord_spec()
    }

    // TODO: All draw_series_impl is overly strict about lifetime, because we don't have stable HKT,
    //       what we can ensure is for all lifetime 'b the element reference &'b E is a iterator
    //       of points reference with the same lifetime.
    //       However, this doesn't work if the coordinate doesn't live longer than the backend,
    //       this is unnecessarily strict
    pub(crate) fn draw_series_impl<B, E, R, S>(
        &mut self,
        series: S,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        B: CoordMapper,
        for<'b> &'b E: PointCollection<'b, CT::From, B>,
        E: Drawable<DB, B>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        Ok(())
    }

    pub(crate) fn alloc_series_anno(&mut self) -> &mut SeriesAnno<'a, DB> {
        let idx = self.series_anno.len();
        self.series_anno.push(SeriesAnno::new());
        &mut self.series_anno[idx]
    }

    /// Draw a data series. A data series in Plotters is abstracted as an iterator of elements
    pub fn draw_series<B, E, R, S>(
        &mut self,
        series: S,
    ) -> Result<&mut SeriesAnno<'a, DB>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        B: CoordMapper,
        for<'b> &'b E: PointCollection<'b, CT::From, B>,
        E: Drawable<DB, B>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        self.draw_series_impl(series)?;
        Ok(self.alloc_series_anno())
    }
}

