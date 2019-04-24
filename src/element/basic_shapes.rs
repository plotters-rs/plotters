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

/// Describe a cross
pub struct Cross<'a, Coord> {
    center: Coord,
    size: u32,
    style: ShapeStyle<'a>,
}

impl <'a, Coord> Cross<'a, Coord> {
    pub fn new(coord:Coord, size:u32, style: ShapeStyle<'a>) -> Self {
        return Self {
            center: coord,
            size,
            style
        };
    }
}

impl <'b, 'a, Coord:'a> PointCollection<'a, Coord> for &'a Cross<'b, Coord> {
    type Borrow  = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        return std::iter::once(&self.center);
    }
}

impl <'a, Coord:'a> Drawable for Cross<'a, Coord> {
    fn draw<DB:DrawingBackend, I:Iterator<Item=BackendCoord>>(&self, mut points:I, backend: &mut DB) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x,y)) = points.next() {
            let size = self.size as i32;
            let (x0,y0) = (x - size, y - size);
            let (x1,y1) = (x + size, y + size);
            backend.draw_line((x0,y0), (x1,y1), &Box::new(self.style.color))?;
            backend.draw_line((x0,y1), (x1,y0), &Box::new(self.style.color))?;
        }
        return Ok(());
    }
}
