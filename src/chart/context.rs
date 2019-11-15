use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;

use super::dual_coord::DualCoordChartContext;
use super::mesh::MeshStyle;
use super::series::SeriesLabelStyle;

use crate::coord::{
    AsRangedCoord, CoordTranslate, MeshLine, Ranged, RangedCoord, ReverseCoordTranslate, Shift,
};
use crate::drawing::backend::{BackendCoord, DrawingBackend};
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::element::{Drawable, DynElement, IntoDynElement, PathElement, PointCollection};
use crate::style::{AsRelative, FontTransform, ShapeStyle, SizeDesc, TextAlignment, TextStyle};

/// The annotations (such as the label of the series, the legend element, etc)
/// When a series is drawn onto a drawing area, an series annotation object
/// is created and a mutable reference is returned.
#[allow(clippy::type_complexity)]
pub struct SeriesAnno<'a, DB: DrawingBackend> {
    label: Option<String>,
    draw_func: Option<Box<dyn Fn(BackendCoord) -> DynElement<'a, DB, BackendCoord> + 'a>>,
    phantom_data: PhantomData<DB>,
}

impl<'a, DB: DrawingBackend> SeriesAnno<'a, DB> {
    pub(crate) fn get_label(&self) -> &str {
        self.label.as_ref().map(|x| x.as_str()).unwrap_or("")
    }

    pub(crate) fn get_draw_func(
        &self,
    ) -> Option<&dyn Fn(BackendCoord) -> DynElement<'a, DB, BackendCoord>> {
        self.draw_func.as_ref().map(|x| x.borrow())
    }

    fn new() -> Self {
        Self {
            label: None,
            draw_func: None,
            phantom_data: PhantomData,
        }
    }

    /// Set the series label
    /// - `label`: The string would be use as label for current series
    pub fn label<L: Into<String>>(&mut self, label: L) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    /// Set the legend element creator function
    /// - `func`: The function use to create the element
    /// *Note*: The creation function uses a shifted pixel-based coordinate system. And place the
    /// point (0,0) to the mid-right point of the shape
    pub fn legend<E: IntoDynElement<'a, DB, BackendCoord>, T: Fn(BackendCoord) -> E + 'a>(
        &mut self,
        func: T,
    ) -> &mut Self {
        self.draw_func = Some(Box::new(move |p| func(p).into_dyn()));
        self
    }
}

/// The context of the chart. This is the core object of Plotters.
/// Any plot/chart is abstracted as this type, and any data series can be placed to the chart
/// context.
pub struct ChartContext<'a, DB: DrawingBackend, CT: CoordTranslate> {
    pub(super) x_label_area: [Option<DrawingArea<DB, Shift>>; 2],
    pub(super) y_label_area: [Option<DrawingArea<DB, Shift>>; 2],
    pub(super) drawing_area: DrawingArea<DB, CT>,
    pub(super) series_anno: Vec<SeriesAnno<'a, DB>>,
    pub(super) drawing_area_pos: (i32, i32),
}

pub struct ChartState<CT: CoordTranslate> {
    drawing_area_pos: (i32, i32),
    drawing_area_size: (u32, u32),
    coord: CT,
}

impl<'a, CT: CoordTranslate + Clone> Clone for ChartState<CT> {
    fn clone(&self) -> Self {
        Self {
            drawing_area_size: self.drawing_area_size,
            drawing_area_pos: self.drawing_area_pos,
            coord: self.coord.clone(),
        }
    }
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate> From<ChartContext<'a, DB, CT>> for ChartState<CT> {
    fn from(chart: ChartContext<'a, DB, CT>) -> ChartState<CT> {
        ChartState {
            drawing_area_pos: chart.drawing_area_pos,
            drawing_area_size: chart.drawing_area.dim_in_pixel(),
            coord: chart.drawing_area.into_coord_spec(),
        }
    }
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate> ChartContext<'a, DB, CT> {
    pub fn into_chart_state(self) -> ChartState<CT> {
        self.into()
    }

    pub fn into_shared_chart_state(self) -> ChartState<Arc<CT>> {
        ChartState {
            drawing_area_pos: self.drawing_area_pos,
            drawing_area_size: self.drawing_area.dim_in_pixel(),
            coord: Arc::new(self.drawing_area.into_coord_spec()),
        }
    }
}

impl<'a, 'b, DB, CT> From<&ChartContext<'a, DB, CT>> for ChartState<CT>
where
    DB: DrawingBackend,
    CT: CoordTranslate + Clone,
{
    fn from(chart: &ChartContext<'a, DB, CT>) -> ChartState<CT> {
        ChartState {
            drawing_area_pos: chart.drawing_area_pos,
            drawing_area_size: chart.drawing_area.dim_in_pixel(),
            coord: chart.drawing_area.as_coord_spec().clone(),
        }
    }
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate + Clone> ChartContext<'a, DB, CT> {
    pub fn to_chart_state(&self) -> ChartState<CT> {
        self.into()
    }
}

impl<CT: CoordTranslate> ChartState<CT> {
    pub fn restore<'a, DB: DrawingBackend>(
        self,
        area: &DrawingArea<DB, Shift>,
    ) -> ChartContext<'a, DB, CT> {
        let area = area
            .clone()
            .shrink(self.drawing_area_pos, self.drawing_area_size);
        ChartContext {
            x_label_area: [None, None],
            y_label_area: [None, None],
            drawing_area: area.apply_coord_spec(self.coord),
            series_anno: vec![],
            drawing_area_pos: self.drawing_area_pos,
        }
    }
}

impl<
        'a,
        DB: DrawingBackend,
        XT: Debug,
        YT: Debug,
        X: Ranged<ValueType = XT>,
        Y: Ranged<ValueType = YT>,
    > ChartContext<'a, DB, RangedCoord<X, Y>>
{
    fn is_overlapping_drawing_area(&self, area: Option<&DrawingArea<DB, Shift>>) -> bool {
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
    /// the function `MeshStyle::draw`
    pub fn configure_mesh<'b>(&'b mut self) -> MeshStyle<'a, 'b, X, Y, DB> {
        let base_tick_size = (5u32).percent().max(5).in_pixels(&self.drawing_area);

        let mut x_tick_size = [base_tick_size, base_tick_size];
        let mut y_tick_size = [base_tick_size, base_tick_size];

        for idx in 0..2 {
            if self.is_overlapping_drawing_area(self.x_label_area[idx].as_ref()) {
                x_tick_size[idx] = -x_tick_size[idx];
            }
            if self.is_overlapping_drawing_area(self.y_label_area[idx].as_ref()) {
                y_tick_size[idx] = -y_tick_size[idx];
            }
        }

        MeshStyle {
            parent_size: self.drawing_area.dim_in_pixel(),
            axis_style: None,
            x_label_offset: 0,
            y_label_offset: 0,
            draw_x_mesh: true,
            draw_y_mesh: true,
            draw_x_axis: true,
            draw_y_axis: true,
            n_x_labels: 10,
            n_y_labels: 10,
            line_style_1: None,
            line_style_2: None,
            x_label_style: None,
            y_label_style: None,
            format_x: &|x| format!("{:?}", x),
            format_y: &|y| format!("{:?}", y),
            target: Some(self),
            _phantom_data: PhantomData,
            x_desc: None,
            y_desc: None,
            axis_desc_style: None,
            x_tick_size,
            y_tick_size,
        }
    }
}

impl<'a, DB: DrawingBackend + 'a, CT: CoordTranslate> ChartContext<'a, DB, CT> {
    /// Configure the styles for drawing series labels in the chart
    pub fn configure_series_labels<'b>(&'b mut self) -> SeriesLabelStyle<'a, 'b, DB, CT> {
        SeriesLabelStyle::new(self)
    }

    /// Get a reference of underlying plotting area
    pub fn plotting_area(&self) -> &DrawingArea<DB, CT> {
        &self.drawing_area
    }
}

impl<'a, DB: DrawingBackend, CT: CoordTranslate> ChartContext<'a, DB, CT> {
    pub fn as_coord_spec(&self) -> &CT {
        self.drawing_area.as_coord_spec()
    }
}

impl<'a, DB: DrawingBackend, CT: ReverseCoordTranslate> ChartContext<'a, DB, CT> {
    /// Convert the chart context into an closure that can be used for coordinate translation
    pub fn into_coord_trans(self) -> impl Fn(BackendCoord) -> Option<CT::From> {
        let coord_spec = self.drawing_area.into_coord_spec();
        move |coord| coord_spec.reverse_translate(coord)
    }
}

impl<'a, DB: DrawingBackend, X: Ranged, Y: Ranged> ChartContext<'a, DB, Arc<RangedCoord<X, Y>>> {
    pub(super) fn draw_series_impl<E, R, S>(
        &mut self,
        series: S,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'b> &'b E: PointCollection<'b, (X::ValueType, Y::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        Ok(())
    }

    pub(super) fn alloc_series_anno(&mut self) -> &mut SeriesAnno<'a, DB> {
        let idx = self.series_anno.len();
        self.series_anno.push(SeriesAnno::new());
        &mut self.series_anno[idx]
    }

    /// Draw a data series. A data series in Plotters is abstracted as an iterator of elements
    pub fn draw_series<E, R, S>(
        &mut self,
        series: S,
    ) -> Result<&mut SeriesAnno<'a, DB>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'b> &'b E: PointCollection<'b, (X::ValueType, Y::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        self.draw_series_impl(series)?;
        Ok(self.alloc_series_anno())
    }
}

impl<'a, DB: DrawingBackend, X: Ranged, Y: Ranged> ChartContext<'a, DB, RangedCoord<X, Y>> {
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

    pub(super) fn draw_series_impl<E, R, S>(
        &mut self,
        series: S,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'b> &'b E: PointCollection<'b, (X::ValueType, Y::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        Ok(())
    }

    pub(super) fn alloc_series_anno(&mut self) -> &mut SeriesAnno<'a, DB> {
        let idx = self.series_anno.len();
        self.series_anno.push(SeriesAnno::new());
        &mut self.series_anno[idx]
    }

    /// Draw a data series. A data series in Plotters is abstracted as an iterator of elements
    pub fn draw_series<E, R, S>(
        &mut self,
        series: S,
    ) -> Result<&mut SeriesAnno<'a, DB>, DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'b> &'b E: PointCollection<'b, (X::ValueType, Y::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        self.draw_series_impl(series)?;
        Ok(self.alloc_series_anno())
    }

    /// The actual function that draws the mesh lines.
    /// It also returns the label that suppose to be there.
    #[allow(clippy::type_complexity)]
    fn draw_mesh_lines<FmtLabel>(
        &mut self,
        (r, c): (usize, usize),
        (x_mesh, y_mesh): (bool, bool),
        mesh_line_style: &ShapeStyle,
        mut fmt_label: FmtLabel,
    ) -> Result<(Vec<(i32, String)>, Vec<(i32, String)>), DrawingAreaErrorKind<DB::ErrorType>>
    where
        FmtLabel: FnMut(&MeshLine<X, Y>) -> Option<String>,
    {
        let mut x_labels = vec![];
        let mut y_labels = vec![];
        self.drawing_area.draw_mesh(
            |b, l| {
                let draw;
                match l {
                    MeshLine::XMesh((x, _), _, _) => {
                        if let Some(label_text) = fmt_label(&l) {
                            x_labels.push((x, label_text));
                        }
                        draw = x_mesh;
                    }
                    MeshLine::YMesh((_, y), _, _) => {
                        if let Some(label_text) = fmt_label(&l) {
                            y_labels.push((y, label_text));
                        }
                        draw = y_mesh;
                    }
                };
                if draw {
                    l.draw(b, mesh_line_style)
                } else {
                    Ok(())
                }
            },
            r,
            c,
        )?;
        Ok((x_labels, y_labels))
    }

    fn draw_axis(
        &self,
        area: &DrawingArea<DB, Shift>,
        axis_style: Option<&ShapeStyle>,
        orientation: (i16, i16),
        inward_labels: bool,
    ) -> Result<Range<i32>, DrawingAreaErrorKind<DB::ErrorType>> {
        let (x0, y0) = self.drawing_area.get_base_pixel();
        let (tw, th) = area.dim_in_pixel();

        let mut axis_range = if orientation.0 == 0 {
            self.drawing_area.get_x_axis_pixel_range()
        } else {
            self.drawing_area.get_y_axis_pixel_range()
        };

        /* At this point, the coordinate system tells us the pixel range
         * after the translation.
         * However, we need to use the logic coordinate system for drawing. */
        if orientation.0 == 0 {
            axis_range.start -= x0;
            axis_range.end -= x0;
        } else {
            axis_range.start -= y0;
            axis_range.end -= y0;
        }

        if let Some(axis_style) = axis_style {
            let mut x0 = if orientation.0 > 0 { 0 } else { tw as i32 };
            let mut y0 = if orientation.1 > 0 { 0 } else { th as i32 };
            let mut x1 = if orientation.0 >= 0 { 0 } else { tw as i32 };
            let mut y1 = if orientation.1 >= 0 { 0 } else { th as i32 };

            if inward_labels {
                if orientation.0 == 0 {
                    if y0 == 0 {
                        y0 = th as i32;
                        y1 = th as i32;
                    } else {
                        y0 = 0;
                        y1 = 0;
                    }
                } else if x0 == 0 {
                    x0 = tw as i32;
                    x1 = tw as i32;
                } else {
                    x0 = 0;
                    x1 = 0;
                }
            }

            if orientation.0 == 0 {
                x0 = axis_range.start;
                x1 = axis_range.end;
            } else {
                y0 = axis_range.start;
                y1 = axis_range.end;
            }

            area.draw(&PathElement::new(
                vec![(x0, y0), (x1, y1)],
                axis_style.clone(),
            ))?;
        }

        Ok(axis_range)
    }

    fn estimate_right_aligned_label_offset(
        &self,
        label_style: &TextStyle,
        labels: &[(i32, String)],
    ) -> i32 {
        labels
            .iter()
            .map(|(_, t)| {
                self.drawing_area
                    .estimate_text_size(t, &label_style.font)
                    .unwrap_or((0, 0))
                    .0
            })
            .max()
            .unwrap_or(0) as i32
    }

    // TODO: consider make this function less complicated
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::cognitive_complexity)]
    fn draw_axis_and_labels(
        &self,
        area: Option<&DrawingArea<DB, Shift>>,
        axis_style: Option<&ShapeStyle>,
        labels: &[(i32, String)],
        label_style: &TextStyle,
        label_offset: i32,
        orientation: (i16, i16),
        axis_desc: Option<(&str, &TextStyle)>,
        tick_size: i32,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        let area = if let Some(target) = area {
            target
        } else {
            return Ok(());
        };

        let (x0, y0) = self.drawing_area.get_base_pixel();
        let (tw, th) = area.dim_in_pixel();

        /* This is the minimal distance from the axis to the box of the labels */
        let label_dist =
            if (tick_size > 0 && orientation.1 > 0) || (tick_size < 0 && orientation.1 < 0) {
                0
            } else {
                tick_size.abs() * 2
            };

        /* All labels are right-aligned. */
        let label_style = &label_style.alignment(TextAlignment::Right);

        /* Draw the axis and get the axis range so that we can do further label
         * and tick mark drawing */
        let axis_range = self.draw_axis(area, axis_style, orientation, tick_size < 0)?;

        /* If the label area is on the right hand side, we should enable the right aligned
         * layout, thus in this case we need to estimate the right most position when all
         * the labels are right aligned */
        let right_alignment = if orientation.0 > 0 && orientation.1 == 0 {
            self.estimate_right_aligned_label_offset(label_style, labels)
        } else {
            0
        };

        /* Then we need to draw the tick mark and the label */
        for (p, t) in labels {
            /* Make sure we are actually in the visible range */
            let rp = if orientation.0 == 0 { *p - x0 } else { *p - y0 };

            if rp < axis_range.start.min(axis_range.end)
                || axis_range.end.max(axis_range.start) < rp
            {
                continue;
            }

            /* Then we need to estimate the text if rendered */
            let (w, h) = self
                .drawing_area
                .estimate_text_size(&t, &label_style.font)
                .unwrap_or((0, 0));

            let (cx, cy) = if tick_size >= 0 {
                match orientation {
                    // Right
                    (dx, dy) if dx > 0 && dy == 0 => {
                        (right_alignment + label_dist - w as i32, *p - y0)
                    }
                    // Left
                    (dx, dy) if dx < 0 && dy == 0 => (tw as i32 - label_dist - w as i32, *p - y0),
                    // Bottom
                    (dx, dy) if dx == 0 && dy > 0 => (*p - x0, label_dist + h as i32),
                    // Top
                    (dx, dy) if dx == 0 && dy < 0 => (*p - x0, th as i32 - label_dist - h as i32),
                    _ => panic!("Bug: Invalid orientation specification"),
                }
            } else {
                match orientation {
                    // Right
                    (dx, dy) if dx > 0 && dy == 0 => {
                        (tw as i32 - right_alignment - label_dist, *p - y0)
                    }
                    // Left
                    (dx, dy) if dx < 0 && dy == 0 => (label_dist, *p - y0),
                    // Bottom
                    (dx, dy) if dx == 0 && dy > 0 => (*p - x0, th as i32 - label_dist - h as i32),
                    // Top
                    (dx, dy) if dx == 0 && dy < 0 => (*p - x0, label_dist + h as i32),
                    _ => panic!("Bug: Invalid orientation specification"),
                }
            };

            let should_draw = if orientation.0 == 0 {
                cx >= 0 && cx + label_offset + w as i32 / 2 <= tw as i32
            } else {
                cy >= 0 && cy + label_offset + h as i32 / 2 <= th as i32
            };

            if should_draw {
                let (text_x, text_y) = if orientation.0 == 0 {
                    (cx - w as i32 / 2 + label_offset, cy)
                } else {
                    (cx, cy - h as i32 / 2 + label_offset)
                };

                area.draw_text(&t, label_style, (text_x, text_y))?;

                if let Some(style) = axis_style {
                    let (kx0, ky0, kx1, ky1) = if tick_size > 0 {
                        match orientation {
                            (dx, dy) if dx > 0 && dy == 0 => (0, *p - y0, tick_size, *p - y0),
                            (dx, dy) if dx < 0 && dy == 0 => {
                                (tw as i32 - tick_size, *p - y0, tw as i32, *p - y0)
                            }
                            (dx, dy) if dx == 0 && dy > 0 => (*p - x0, 0, *p - x0, tick_size),
                            (dx, dy) if dx == 0 && dy < 0 => {
                                (*p - x0, th as i32 - tick_size, *p - x0, th as i32)
                            }
                            _ => panic!("Bug: Invalid orientation specification"),
                        }
                    } else {
                        match orientation {
                            (dx, dy) if dx > 0 && dy == 0 => {
                                (tw as i32, *p - y0, tw as i32 + tick_size, *p - y0)
                            }
                            (dx, dy) if dx < 0 && dy == 0 => (0, *p - y0, -tick_size, *p - y0),
                            (dx, dy) if dx == 0 && dy > 0 => {
                                (*p - x0, th as i32, *p - x0, th as i32 + tick_size)
                            }
                            (dx, dy) if dx == 0 && dy < 0 => (*p - x0, 0, *p - x0, -tick_size),
                            _ => panic!("Bug: Invalid orientation specification"),
                        }
                    };
                    let line = PathElement::new(vec![(kx0, ky0), (kx1, ky1)], style.clone());
                    area.draw(&line)?;
                }
            }
        }

        if let Some((text, style)) = axis_desc {
            let actual_style = if orientation.0 == 0 {
                style.clone()
            } else if orientation.0 == -1 {
                style.transform(FontTransform::Rotate270)
            } else {
                style.transform(FontTransform::Rotate90)
            };

            let (w, h) = self
                .drawing_area
                .estimate_text_size(text, &actual_style.font)
                .unwrap_or((0, 0));

            let (x0, y0) = match orientation {
                (dx, dy) if dx > 0 && dy == 0 => (tw - w, (th - h) / 2),
                (dx, dy) if dx < 0 && dy == 0 => (0, (th - h) / 2),
                (dx, dy) if dx == 0 && dy > 0 => ((tw - w) / 2, th - h),
                (dx, dy) if dx == 0 && dy < 0 => ((tw - w) / 2, 0),
                _ => panic!("Bug: Invalid orientation specification"),
            };

            area.draw_text(&text, &actual_style, (x0 as i32, y0 as i32))?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_mesh<FmtLabel>(
        &mut self,
        (r, c): (usize, usize),
        mesh_line_style: &ShapeStyle,
        x_label_style: &TextStyle,
        y_label_style: &TextStyle,
        fmt_label: FmtLabel,
        x_mesh: bool,
        y_mesh: bool,
        x_label_offset: i32,
        y_label_offset: i32,
        x_axis: bool,
        y_axis: bool,
        axis_style: &ShapeStyle,
        axis_desc_style: &TextStyle,
        x_desc: Option<String>,
        y_desc: Option<String>,
        x_tick_size: [i32; 2],
        y_tick_size: [i32; 2],
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        FmtLabel: FnMut(&MeshLine<X, Y>) -> Option<String>,
    {
        let (x_labels, y_labels) =
            self.draw_mesh_lines((r, c), (x_mesh, y_mesh), mesh_line_style, fmt_label)?;

        for idx in 0..2 {
            self.draw_axis_and_labels(
                self.x_label_area[idx].as_ref(),
                if x_axis { Some(axis_style) } else { None },
                &x_labels[..],
                x_label_style,
                x_label_offset,
                (0, -1 + idx as i16 * 2),
                x_desc.as_ref().map(|desc| (&desc[..], axis_desc_style)),
                x_tick_size[idx],
            )?;

            self.draw_axis_and_labels(
                self.y_label_area[idx].as_ref(),
                if y_axis { Some(axis_style) } else { None },
                &y_labels[..],
                y_label_style,
                y_label_offset,
                (-1 + idx as i16 * 2, 0),
                y_desc.as_ref().map(|desc| (&desc[..], axis_desc_style)),
                y_tick_size[idx],
            )?;
        }

        Ok(())
    }

    /// Convert this chart context into a dual axis chart context
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
        RangedCoord<X, Y>,
        RangedCoord<SX::CoordDescType, SY::CoordDescType>,
    > {
        let mut pixel_range = self.drawing_area.get_pixel_range();
        pixel_range.1 = pixel_range.1.end..pixel_range.1.start;

        DualCoordChartContext::new(self, RangedCoord::new(x_coord, y_coord, pixel_range))
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_chart_context() {
        let drawing_area = create_mocked_drawing_area(200, 200, |_| {});

        drawing_area.fill(&WHITE).expect("Fill");

        let mut chart = ChartBuilder::on(&drawing_area)
            .caption("Test Title", ("serif", 10))
            .x_label_area_size(20)
            .y_label_area_size(20)
            .set_label_area_size(LabelAreaPosition::Top, 20)
            .set_label_area_size(LabelAreaPosition::Right, 20)
            .build_ranged(0..10, 0..10)
            .expect("Create chart")
            .set_secondary_coord(0.0..1.0, 0.0..1.0);

        chart
            .configure_mesh()
            .x_desc("X")
            .y_desc("Y")
            .draw()
            .expect("Draw mesh");
        chart
            .configure_secondary_axes()
            .x_desc("X")
            .y_desc("Y")
            .draw()
            .expect("Draw Secondary axes");

        chart
            .draw_series(std::iter::once(Circle::new((5, 5), 5, &RED)))
            .expect("Drawing error");
        chart
            .draw_secondary_series(std::iter::once(Circle::new((0.3, 0.8), 5, &GREEN)))
            .expect("Drawing error")
            .label("Test label")
            .legend(|(x, y)| Rectangle::new([(x - 10, y - 5), (x, y + 5)], &GREEN));

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperMiddle)
            .draw()
            .expect("Drawing error");
    }
}
