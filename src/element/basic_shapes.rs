/// Define the basic shapes
use crate::style::ShapeStyle;
use crate::drawing::backend::{DrawingBackend, DrawingErrorKind, BackendCoord};
use super::{Drawable, PointCollection};

/// Describe a path
pub struct Path<'a, Coord> {
    points: Vec<Coord>, 
    style: ShapeStyle<'a>,
}
impl <'a, Coord> Path<'a, Coord> {
    pub fn new<P:Into<Vec<Coord>>>(points:P, style: ShapeStyle<'a>) -> Self {
        return Self{
            points: points.into(),
            style
        };
    }
}

impl <'b, 'a, Coord:'a> PointCollection<'a, Coord> for &'a Path<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        return &self.points;
    }
}

impl <'a, Coord:'a> Drawable for Path<'a, Coord> {
    fn draw<DB:DrawingBackend, I:Iterator<Item=BackendCoord>>(&self, points:I, backend: &mut DB) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        return backend.draw_path(points, &Box::new(self.style.color));
    }
}

