use std::borrow::Borrow;
use std::ops::Range;

use crate::chart::{ChartContext, DualCoordChartContext, MeshStyle, SeriesAnno};
use crate::coord::{
    cartesian::Cartesian2d,
    ranged1d::{AsRangedCoord, Ranged, ValueFormatter},
    Shift,
};
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::element::{CoordMapper, Drawable, PointCollection};
use crate::style::Color;
use plotters_backend::{BackendCoord, DrawingBackend, Interpolation};

mod draw_impl;

impl<'a, DB, XT, YT, X, Y> ChartContext<'a, DB, Cartesian2d<X, Y>>
where
    DB: DrawingBackend,
    X: Ranged<ValueType = XT> + ValueFormatter<XT>,
    Y: Ranged<ValueType = YT> + ValueFormatter<YT>,
{
    pub(crate) fn is_overlapping_drawing_area(
        &self,
        area: Option<&DrawingArea<DB, Shift>>,
    ) -> bool {
        if let Some(area) = area {
            let (x0, y0) = area.get_base_pixel();
            let (w, h) = area.dim_in_pixel();
            let (x1, y1) = (x0 + w as i32, y0 + h as i32);
            let (dx0, dy0) = self.drawing_area.get_base_pixel();
            let (w, h) = self.drawing_area.dim_in_pixel();
            let (dx1, dy1) = (dx0 + w as i32, dy0 + h as i32);

            let (ox0, ox1) = (x0.max(dx0), x1.min(dx1));
            let (oy0, oy1) = (y0.max(dy0), y1.min(dy1));

            ox1 > ox0 && oy1 > oy0
        } else {
            false
        }
    }

    /// Initialize a mesh configuration object and mesh drawing can be finalized by calling
    /// the function `MeshStyle::draw`.
    pub fn configure_mesh(&mut self) -> MeshStyle<'a, '_, X, Y, DB> {
        MeshStyle::new(self)
    }

    /// Draw a series while emitting semantic contexts for every element.
    ///
    /// This is the tooltip-aware counterpart of [`ChartContext::draw_series`].
    /// Each element in the iterator must yield guest coordinates `(XT, YT)`.
    /// The method inspects each element's points:
    ///
    /// - **Single-point** elements (markers, circles) are wrapped in a
    /// [`ElementContext::DataPoint`] context with formatted x/y labels.
    /// - **Multi-point** elements (lines, paths) are wrapped in a [`ElementContext::DataLine`]
    /// context that carries discrete interpolation data (every vertex with its formatted label).
    ///
    /// The whole series is wrapped in a [`ElementContext::DataSeries`] context so interactive
    /// backends can group and style them.
    ///
    /// `series_color` and `series_label` describe the series metadata forwarded to the backend.
    pub fn draw_series_with_tooltips<B, E, R, S, C>(
        &mut self,
        series: S,
        series_color: &C,
        series_label: &str,
    ) -> Result<&mut SeriesAnno<'a, DB>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        B: CoordMapper,
        for<'b> &'b E: PointCollection<'b, (XT, YT), B>,
        E: Drawable<DB, B>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
        C: Color,
    {
        let series_id = self.next_series_id;
        self.next_series_id += 1;

        let bc = series_color.to_backend_color();
        self.drawing_area
            .begin_context(plotters_backend::ElementContext::DataSeries {
                id: series_id,
                color: bc,
                label: series_label.to_string(),
            })?;

        let x_spec = self.drawing_area.as_coord_spec().x_spec();
        let y_spec = self.drawing_area.as_coord_spec().y_spec();

        for element in series {
            let elem = element.borrow();

            // Collect all guest points and map them to backend coords + labels
            let mapped: Vec<_> = elem
                .point_iter()
                .into_iter()
                .map(|pt| {
                    let guest = pt.borrow();
                    let xl = X::format_ext(x_spec, &guest.0);
                    let yl = Y::format_ext(y_spec, &guest.1);
                    let coord = self.drawing_area.map_coordinate(guest);
                    (coord, xl, yl)
                })
                .collect();

            let opened = if mapped.len() <= 1 {
                // Single-point element->DataPoint context
                if let Some((coord, xl, yl)) = mapped.into_iter().next() {
                    self.drawing_area.begin_context(
                        plotters_backend::ElementContext::DataPoint {
                            coord,
                            x_label: xl,
                            y_label: yl,
                            series_id,
                        },
                    )?;
                    true
                } else {
                    false
                }
            } else {
                // Multi-point element -> DataLine context with discrete interpolation (every vertex
                // carries a formatted label).
                let x_points: Vec<_> = mapped.iter().map(|(c, xl, _)| (c.0, xl.clone())).collect();
                let y_points: Vec<_> = mapped.iter().map(|(c, _, yl)| (c.1, yl.clone())).collect();
                self.drawing_area
                    .begin_context(plotters_backend::ElementContext::DataLine {
                        x_interpolation: Interpolation::Discrete { points: x_points },
                        y_interpolation: Interpolation::Discrete { points: y_points },
                        series_id,
                    })?;
                true
            };
            self.drawing_area.draw(elem)?;
            if opened {
                self.drawing_area.end_context()?;
            }
        }
        self.drawing_area.end_context()?; // close DataSeries
        Ok(self.alloc_series_anno())
    }
}

impl<'a, DB: DrawingBackend, X: Ranged, Y: Ranged> ChartContext<'a, DB, Cartesian2d<X, Y>> {
    /// Get the range of X axis
    pub fn x_range(&self) -> Range<X::ValueType> {
        self.drawing_area.get_x_range()
    }

    /// Get range of the Y axis
    pub fn y_range(&self) -> Range<Y::ValueType> {
        self.drawing_area.get_y_range()
    }

    /// Maps the coordinate to the backend coordinate. This is typically used
    /// with an interactive chart.
    pub fn backend_coord(&self, coord: &(X::ValueType, Y::ValueType)) -> BackendCoord {
        self.drawing_area.map_coordinate(coord)
    }
}

impl<'a, DB: DrawingBackend, X: Ranged, Y: Ranged> ChartContext<'a, DB, Cartesian2d<X, Y>> {
    /// Convert this chart context into a dual axis chart context and attach a second coordinate spec
    /// on the chart context. For more detailed information, see documentation for [struct DualCoordChartContext](struct.DualCoordChartContext.html)
    ///
    /// - `x_coord`: The coordinate spec for the X axis
    /// - `y_coord`: The coordinate spec for the Y axis
    /// - **returns** The newly created dual spec chart context
    #[allow(clippy::type_complexity)]
    pub fn set_secondary_coord<SX: AsRangedCoord, SY: AsRangedCoord>(
        self,
        x_coord: SX,
        y_coord: SY,
    ) -> DualCoordChartContext<
        'a,
        DB,
        Cartesian2d<X, Y>,
        Cartesian2d<SX::CoordDescType, SY::CoordDescType>,
    > {
        let mut pixel_range = self.drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        DualCoordChartContext::new(self, Cartesian2d::new(x_coord, y_coord, pixel_range))
    }
}
