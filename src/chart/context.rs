use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

use super::mesh::MeshStyle;

use crate::coord::{CoordTranslate, MeshLine, Ranged, RangedCoord, ReverseCoordTranslate, Shift};
use crate::drawing::backend::{BackendCoord, DrawingBackend};
use crate::drawing::{DrawingArea, DrawingAreaErrorKind};
use crate::element::{Drawable, Path, PointCollection};
use crate::style::{FontTransform, ShapeStyle, TextStyle};

/// The context of the chart. This is the core object of Plotters.
/// Any plot/chart is abstracted as this type, and any data series can be placed to the chart
/// context.
pub struct ChartContext<DB: DrawingBackend, CT: CoordTranslate> {
    pub(super) x_label_area: Option<DrawingArea<DB, Shift>>,
    pub(super) y_label_area: Option<DrawingArea<DB, Shift>>,
    pub(super) drawing_area: DrawingArea<DB, CT>,
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
            x_desc: None,
            y_desc: None,
            axis_desc_style: None,
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

    /// Maps the coordinate to the backend coordinate. This is typically used
    /// with an interactive chart.
    pub fn backend_coord(&self, coord: &(X::ValueType, Y::ValueType)) -> BackendCoord {
        self.drawing_area.map_coordinate(coord)
    }

    /// Draw a data series. A data series in Plotters is abstracted as an iterator of elements
    pub fn draw_series<E, R, S>(&self, series: S) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        for<'a> &'a E: PointCollection<'a, (X::ValueType, Y::ValueType)>,
        E: Drawable<DB>,
        R: Borrow<E>,
        S: IntoIterator<Item = R>,
    {
        for element in series {
            self.drawing_area.draw(element.borrow())?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_mesh<FmtLabel>(
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
        axis_desc_style: &TextStyle,
        x_desc: Option<String>,
        y_desc: Option<String>,
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
            let (tw, th) = xl.dim_in_pixel();
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

            if let Some(ref text) = x_desc {
                let (w, h) = label_style.font.box_size(text).unwrap_or((0, 0));

                let left = (tw - w) / 2;
                let top = th - h;

                xl.draw_text(&text, axis_desc_style, (left as i32, top as i32))?;
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

            if let Some(ref text) = y_desc {
                let (w, _) = label_style.font.box_size(text).unwrap_or((0, 0));

                let top = (th - w) / 2;

                let mut y_style = axis_desc_style.clone();
                let y_font = axis_desc_style.font.transform(FontTransform::Rotate270);
                y_style.font = &y_font;

                yl.draw_text(&text, &y_style, (0, top as i32))?;
            }
        }

        Ok(())
    }
}
