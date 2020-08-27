use super::*;
use super::{Drawable, PointCollection};
use crate::style::{ShapeStyle, SizeDesc};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

/// The element that used to describe a point
pub trait PointElement<Coord, Size: SizeDesc> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self;
}

/// Describe a cross
pub struct Cross<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> Cross<Coord, Size> {
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a, Size: SizeDesc> PointCollection<'a, Coord> for &'a Cross<Coord, Size> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<Coord, DB: DrawingBackend, Size: SizeDesc> Drawable<DB> for Cross<Coord, Size> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            let size = self.size.in_pixels(&ps);
            let (x0, y0) = (x - size, y - size);
            let (x1, y1) = (x + size, y + size);
            backend.draw_line((x0, y0), (x1, y1), &self.style)?;
            backend.draw_line((x0, y1), (x1, y0), &self.style)?;
        }
        Ok(())
    }
}

/// Describe a triangle marker
pub struct TriangleMarker<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> TriangleMarker<Coord, Size> {
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a, Size: SizeDesc> PointCollection<'a, Coord> for &'a TriangleMarker<Coord, Size> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<Coord, DB: DrawingBackend, Size: SizeDesc> Drawable<DB> for TriangleMarker<Coord, Size> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            let size = self.size.in_pixels(&ps);
            let points = [-90, -210, -330]
                .iter()
                .map(|deg| f64::from(*deg) * std::f64::consts::PI / 180.0)
                .map(|rad| {
                    (
                        (rad.cos() * f64::from(size) + f64::from(x)).ceil() as i32,
                        (rad.sin() * f64::from(size) + f64::from(y)).ceil() as i32,
                    )
                });
            backend.fill_polygon(points, &self.style.color)?;
        }
        Ok(())
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Cross<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for TriangleMarker<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Circle<Coord, Size> {
    fn make_point(pos: Coord, size: Size, style: ShapeStyle) -> Self {
        Self::new(pos, size, style)
    }
}

impl<Coord, Size: SizeDesc> PointElement<Coord, Size> for Pixel<Coord> {
    fn make_point(pos: Coord, _: Size, style: ShapeStyle) -> Self {
        Self::new(pos, style)
    }
}
