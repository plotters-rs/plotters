/*!
The high-level plotting abstractions.

`ChartBuilder` can be used to create a `ChartContext`.
We are able to draw multiple series in the chart.
A chart can be build on top of any drawing area based on a pixel coordinate.
*/

use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

use crate::coord::{
    AsRangedCoord, CoordTranslate, MeshLine, Ranged, RangedCoord, ReverseCoordTranslate, Shift,
};

use crate::drawing::backend::BackendCoord;
use crate::drawing::backend::DrawingBackend;
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::element::{Drawable, Path, PointCollection, Rectangle};
use crate::style::{FontDesc, Mixable, RGBColor, ShapeStyle, TextStyle};

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

    /// Builder the chart with a ranged coordinate system. The function will returns a chart
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
            series_area: None,
            drawing_area: drawing_area.apply_coord_spec(RangedCoord::new(
                x_spec,
                y_spec,
                pixel_range,
            )),
        })
    }
}

/// The context of the chart. This is the core object of Plotters.
/// Any plot/chart is abstracted as this type, and any data series can be placed to the chart
/// context.
pub struct ChartContext<DB: DrawingBackend, CT: CoordTranslate> {
    x_label_area: Option<DrawingArea<DB, Shift>>,
    y_label_area: Option<DrawingArea<DB, Shift>>,
    series_area: Option<DrawingArea<DB, Shift>>,
    drawing_area: DrawingArea<DB, CT>,
}

/// The struct that is used for tracking the configuration of a mesh of any chart
pub struct MeshStyle<'a, X: Ranged, Y: Ranged, DB>
where
    DB: DrawingBackend,
{
    draw_x_mesh: bool,
    draw_y_mesh: bool,
    draw_x_axis: bool,
    draw_y_axis: bool,
    x_label_offset: i32,
    n_x_labels: usize,
    n_y_labels: usize,
    line_style_1: Option<ShapeStyle<'a>>,
    line_style_2: Option<ShapeStyle<'a>>,
    axis_style: Option<ShapeStyle<'a>>,
    label_style: Option<TextStyle<'a>>,
    format_x: &'a dyn Fn(&X::ValueType) -> String,
    format_y: &'a dyn Fn(&Y::ValueType) -> String,
    target: Option<&'a mut ChartContext<DB, RangedCoord<X, Y>>>,
    _pahtom_data: PhantomData<(X, Y)>,
}

impl<'a, X, Y, DB> MeshStyle<'a, X, Y, DB>
where
    X: Ranged,
    Y: Ranged,
    DB: DrawingBackend,
{
    /// The offset of x labels. This is used when we want to place the label in the middle of
    /// the grid. This is useful if we are drawing a histogram
    /// - `value`: The offset in pixel
    pub fn x_label_offset(&mut self, value: i32) -> &mut Self {
        self.x_label_offset = value;
        self
    }

    /// Disable the mesh for the x axis.
    pub fn disable_x_mesh(&mut self) -> &mut Self {
        self.draw_x_mesh = false;
        self
    }

    /// Disable the mesh for the y axis
    pub fn disable_y_mesh(&mut self) -> &mut Self {
        self.draw_y_mesh = false;
        self
    }

    /// Disable drawing the X axis
    pub fn disable_x_axis(&mut self) -> &mut Self {
        self.draw_x_axis = false;
        self
    }

    /// Disable drawing the Y axis
    pub fn disable_y_axis(&mut self) -> &mut Self {
        self.draw_y_axis = false;
        self
    }

    /// Set the style definition for the axis
    /// - `style`: The style for the axis
    pub fn axis_style<T: Into<ShapeStyle<'a>>>(&mut self, style: T) -> &mut Self {
        self.axis_style = Some(style.into());
        self
    }
    /// Set how many labels for the X axis at most
    /// - `value`: The maximum desired number of labels in the X axis
    pub fn x_labels(&mut self, value: usize) -> &mut Self {
        self.n_x_labels = value;
        self
    }

    /// Set how many label for the Y axis at most
    /// - `value`: The maximum desired number of labels in the Y axis
    pub fn y_labels(&mut self, value: usize) -> &mut Self {
        self.n_y_labels = value;
        self
    }

    /// Set the style for the coarse grind grid
    /// - `style`: This is the fcoarse grind grid style
    pub fn line_style_1<T: Into<ShapeStyle<'a>>>(&mut self, style: T) -> &mut Self {
        self.line_style_1 = Some(style.into());
        self
    }

    /// Set the style for the fine grind grid
    /// - `style`: The fine grind grid style
    pub fn line_style_2<T: Into<ShapeStyle<'a>>>(&mut self, style: T) -> &mut Self {
        self.line_style_2 = Some(style.into());
        self
    }

    /// Set the style of the label text
    /// - `style`: The text style that would be applied to the labels
    pub fn label_style<T: Into<TextStyle<'a>>>(&mut self, style: T) -> &mut Self {
        self.label_style = Some(style.into());
        self
    }

    /// Set the formatter function for the X label text
    /// - `fmt`: The formatter function
    pub fn x_label_formatter(&mut self, fmt: &'a dyn Fn(&X::ValueType) -> String) -> &mut Self {
        self.format_x = fmt;
        self
    }

    /// Set the formatter function for the Y label text
    /// - `fmt`: The formatter function
    pub fn y_label_formatter(&mut self, fmt: &'a dyn Fn(&Y::ValueType) -> String) -> &mut Self {
        self.format_y = fmt;
        self
    }

    /// Draw the configured mesh on the target plot
    pub fn draw(&mut self) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        let mut target = None;
        std::mem::swap(&mut target, &mut self.target);
        let target = target.unwrap();

        let default_mesh_color_1 = RGBColor(0, 0, 0).mix(0.2);
        let default_mesh_color_2 = RGBColor(0, 0, 0).mix(0.1);
        let default_axis_color = RGBColor(0, 0, 0);
        let default_label_font = FontDesc::new("Arial", 12.0);

        let mesh_style_1 = self
            .line_style_1
            .clone()
            .unwrap_or_else(|| (&default_mesh_color_1).into());
        let mesh_style_2 = self
            .line_style_2
            .clone()
            .unwrap_or_else(|| (&default_mesh_color_2).into());
        let axis_style = self
            .axis_style
            .clone()
            .unwrap_or_else(|| (&default_axis_color).into());

        let label_style =
            unsafe { std::mem::transmute::<_, Option<TextStyle>>(self.label_style.clone()) }
                .unwrap_or_else(|| (&default_label_font).into());

        target.draw_mesh(
            (self.n_y_labels * 10, self.n_x_labels * 10),
            &mesh_style_2,
            &label_style,
            |_| None,
            self.draw_x_mesh,
            self.draw_y_mesh,
            self.x_label_offset,
            false,
            false,
            &axis_style,
        )?;

        target.draw_mesh(
            (self.n_y_labels, self.n_x_labels),
            &mesh_style_1,
            &label_style,
            |m| match m {
                MeshLine::XMesh(_, _, v) => Some((self.format_x)(v)),
                MeshLine::YMesh(_, _, v) => Some((self.format_y)(v)),
            },
            self.draw_x_mesh,
            self.draw_y_mesh,
            self.x_label_offset,
            self.draw_x_axis,
            self.draw_y_axis,
            &axis_style,
        )
    }
}

impl<
        DB: DrawingBackend,
        XT: Debug,
        YT: Debug,
        X: Ranged<ValueType = XT>,
        Y: Ranged<ValueType = YT>,
    > ChartContext<DB, RangedCoord<X, Y>>
{
    /// Initialize a mesh configuration object and mesh drawing can be finalized by calling
    /// the function `MeshStyle::draw`
    pub fn configure_mesh(&mut self) -> MeshStyle<X, Y, DB> {
        MeshStyle {
            axis_style: None,
            x_label_offset: 0,
            draw_x_mesh: true,
            draw_y_mesh: true,
            draw_x_axis: true,
            draw_y_axis: true,
            n_x_labels: 10,
            n_y_labels: 10,
            line_style_1: None,
            line_style_2: None,
            label_style: None,
            format_x: &|x| format!("{:?}", x),
            format_y: &|y| format!("{:?}", y),
            target: Some(self),
            _pahtom_data: PhantomData,
        }
    }
}

impl<DB: DrawingBackend, CT: ReverseCoordTranslate> ChartContext<DB, CT> {
    /// Convert the chart context into an closure that can be used for coordinate translation
    pub fn into_coord_trans(self) -> impl Fn(BackendCoord) -> Option<CT::From> {
        let coord_spec = self.drawing_area.into_coord_spec();
        move |coord| coord_spec.reverse_translate(coord)
    }
}

impl<DB: DrawingBackend, X: Ranged, Y: Ranged> ChartContext<DB, RangedCoord<X, Y>> {
    /// Get the range of X axis
    pub fn x_range(&self) -> Range<X::ValueType> {
        self.drawing_area.get_x_range()
    }

    /// Get range of the Y axis
    pub fn y_range(&self) -> Range<Y::ValueType> {
        self.drawing_area.get_y_range()
    }

    /// Get a reference of underlying plotting area
    pub fn plotting_area(&self) -> &DrawingArea<DB, RangedCoord<X, Y>> {
        &self.drawing_area
    }

    /// Defines a series label area
    pub fn define_series_label_area<'a, S: Into<ShapeStyle<'a>>>(
        &mut self,
        pos: (u32, u32),
        size: (u32, u32),
        bg_style: S,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        // TODO: we should be able to draw the label
        self.series_area = Some(self.drawing_area.strip_coord_spec().shrink(pos, size));
        let element = Rectangle::new([(0, 0), (size.0 as i32, size.1 as i32)], bg_style.into());
        self.series_area.as_ref().unwrap().draw(&element)
    }

    /// Maps the coordinate to the backend coordinate. This is typically used
    /// with an interactive chart.
    pub fn backend_coord(&self, coord: &(X::ValueType, Y::ValueType)) -> BackendCoord {
        self.drawing_area.map_coordinate(coord)
    }

    /// Draw a data series. A data series in Plotters is abstracted as an iterator of elements
    pub fn draw_series<E, R, S>(&self, series: S) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'a> &'a E: PointCollection<'a, (X::ValueType, Y::ValueType)>,
        E: Drawable,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_mesh<FmtLabel>(
        &mut self,
        (r, c): (usize, usize),
        mesh_line_style: &ShapeStyle,
        label_style: &TextStyle,
        mut fmt_label: FmtLabel,
        x_mesh: bool,
        y_mesh: bool,
        x_label_offset: i32,
        x_axis: bool,
        y_axis: bool,
        axis_style: &ShapeStyle,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
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

        let (x0, y0) = self.drawing_area.get_base_pixel();

        if let Some(ref xl) = self.x_label_area {
            let (tw, _) = xl.dim_in_pixel();
            if x_axis {
                xl.draw(&Path::new(vec![(0, 0), (tw as i32, 0)], axis_style.clone()))?;
            }
            for (p, t) in x_labels {
                let (w, _) = label_style.font.box_size(&t).unwrap_or((0, 0));

                if p - x0 + x_label_offset > 0 && p - x0 + x_label_offset + w as i32 / 2 < tw as i32
                {
                    if x_axis {
                        xl.draw(&Path::new(
                            vec![(p - x0, 0), (p - x0, 5)],
                            axis_style.clone(),
                        ))?;
                    }
                    xl.draw_text(
                        &t,
                        label_style,
                        (p - x0 - w as i32 / 2 + x_label_offset, 10),
                    )?;
                }
            }
        }

        if let Some(ref yl) = self.y_label_area {
            let (tw, th) = yl.dim_in_pixel();
            if y_axis {
                yl.draw(&Path::new(
                    vec![(tw as i32, 0), (tw as i32, th as i32)],
                    axis_style.clone(),
                ))?;
            }
            for (p, t) in y_labels {
                let (w, h) = label_style.font.box_size(&t).unwrap_or((0, 0));
                if p - y0 >= 0 && p - y0 - h as i32 / 2 <= th as i32 {
                    yl.draw_text(
                        &t,
                        label_style,
                        (tw as i32 - w as i32 - 10, p - y0 - h as i32 / 2),
                    )?;
                    if y_axis {
                        yl.draw(&Path::new(
                            vec![(tw as i32 - 5, p - y0), (tw as i32, p - y0)],
                            axis_style.clone(),
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }
}
