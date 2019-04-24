use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::ShapeStyle;

/// The element that used to describe a point
pub trait PointElement<'a, Coord> {
    fn make_point(pos: Coord, size: u32, style: ShapeStyle<'a>) -> Self;
}

/// Describe a cross
pub struct Cross<'a, Coord> {
    center: Coord,
    size: u32,
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Cross<'a, Coord> {
    pub fn new(coord: Coord, size: u32, style: ShapeStyle<'a>) -> Self {
        return Self {
            center: coord,
            size,
            style,
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Cross<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        return std::iter::once(&self.center);
    }
}

impl<'a, Coord: 'a> Drawable for Cross<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            let size = self.size as i32;
            let (x0, y0) = (x - size, y - size);
            let (x1, y1) = (x + size, y + size);
            backend.draw_line((x0, y0), (x1, y1), &Box::new(self.style.color))?;
            backend.draw_line((x0, y1), (x1, y0), &Box::new(self.style.color))?;
        }
        return Ok(());
    }
}
impl<'a, Coord> PointElement<'a, Coord> for Cross<'a, Coord> {
    fn make_point(pos: Coord, size: u32, style: ShapeStyle<'a>) -> Self {
        return Self::new(pos, size, style);
    }
}
