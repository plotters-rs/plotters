use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
/// Define the basic shapes
use crate::style::ShapeStyle;

/// Describe a path
pub struct Path<'a, Coord> {
    points: Vec<Coord>,
    style: ShapeStyle<'a>,
}
impl<'a, Coord> Path<'a, Coord> {
    pub fn new<P: Into<Vec<Coord>>, S: Into<ShapeStyle<'a>>>(points: P, style: S) -> Self {
        return Self {
            points: points.into(),
            style: style.into(),
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Path<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        return &self.points;
    }
}

impl<'a, Coord: 'a> Drawable for Path<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        return backend.draw_path(points, &Box::new(self.style.color));
    }
}
/*
pub struct Rectangle<'a, Coord, C:Color> {
    points:[Coord;2],
    style: ShapeStyle<'a>
}

impl <Coord, C:Color> Rectangle<Coord, C> {
    pub fn new(points:[Coord;2], color:C, filled:bool) -> Self {
        return Self { points, color, filled };
    }
}

impl <'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Rectangle<

impl <'a, Coord:'a, C:Color> Element<'a, Coord> for Rectangle<Coord, C> where Self:'a {
    type Points = &'a [Coord];

    fn points(&'a self) -> &'a [Coord] {
        return &self.points;
    }

    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, points:I, dc: &mut DC) -> Result<(), DC::ErrorType> {
        let points:Vec<_> = points.into_iter().map(|(a,b)|(a as i32, b as i32)).take(2).collect();

        dc.draw_rect(points[0], points[1], &self.color, self.filled)
    }
}

pub struct Text<'b, 'a:'b , Coord, C:Color> {
    text: &'b str,
    font: &'b FontDesc<'a>,
    coord: Coord,
    color: C,
}
impl <'b, 'a:'b, Coord, C:Color> Text<'b, 'a, Coord, C> {
    pub fn new(text: &'b str, font: &'b FontDesc<'a>, coord: Coord, color:C) -> Self {
        return Self {text, font, coord, color};
    }

}
impl <'b, 'a:'b, Coord:'b, C:Color> Element<'b, Coord> for Text<'b, 'a, Coord, C> {
    type Points = Once<&'b Coord>;

    fn points(&'b self) -> Self::Points {
        return once(&self.coord);
    }

    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, points:I, dc: &mut DC) -> Result<(), DC::ErrorType> {
        let pos = points.into_iter().next().unwrap();

        dc.draw_text(self.text, self.font, (pos.0 as i32, pos.1 as i32), &self.color)
    }
}

pub struct Circle<Coord, C:Color> {
    coord: Coord,
    radius: u32,
    color: C,
    filled: bool,
}

impl <Coord, C:Color> Circle<Coord, C> {
    pub fn new(coord:Coord, radius:u32, color:C, filled: bool) -> Self {
        return Self {coord, radius, color, filled};
    }
}

impl <'a, Coord:'a, C:Color> Element<'a, Coord> for Circle<Coord, C> where Self:'a {
    type Points = Once<&'a Coord>;

    fn points(&'a self) -> Self::Points {
        return once(&self.coord);
    }

    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, points:I, dc: &mut DC) -> Result<(), DC::ErrorType> {
        let pos = points.into_iter().next().unwrap();
        dc.draw_circle((pos.0 as i32, pos.1 as i32), self.radius, &self.color, self.filled)
    }
}
*/
