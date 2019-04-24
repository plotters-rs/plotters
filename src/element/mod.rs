/// Defines the drawing elements, which is the high-level drawing interface
use crate::drawing::backend::{DrawingBackend, DrawingErrorKind, BackendCoord};
use std::borrow::Borrow;

/// The trait that represents an element drawed on the canvas
pub trait PointCollection<'a, Coord> {
    /// The item in point iterator 
    type Borrow : Borrow<Coord>;
    
    /// The point iterator
    type IntoIter : IntoIterator<Item = Self::Borrow>;

    /// framework to do the coordinate mapping
    fn point_iter(self) -> Self::IntoIter;
}

pub trait Drawable {
    /// Actually draws the element. The key points is already translated into the 
    /// image cooridnate and can be used by DC directly
    fn draw<DB:DrawingBackend, I:Iterator<Item=BackendCoord>>(&self, pos:I, backend: &mut DB) -> Result<(), DrawingErrorKind<DB::ErrorType>>;
}

mod basic_shapes;
pub use basic_shapes::*;

/*
impl <'a, Coord:'a, C:Color> Element<'a, Coord> for Path<Coord, C> where Self:'a {
    type Points = &'a [Coord];

    fn points(&'a self) -> &'a [Coord] {
        return &self.points;
    }

    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, points:I, dc: &mut DC) -> Result<(), DC::ErrorType> {
        dc.draw_path(points.map(|(a,b)| (a as i32, b as i32)), &self.color)
    }
}

pub struct Rectangle<Coord, C:Color> {
    points:[Coord;2], 
    color: C, 
    filled: bool
}
impl <Coord, C:Color> Rectangle<Coord, C> {
    pub fn new(points:[Coord;2], color:C, filled:bool) -> Self {
        return Self { points, color, filled };
    }
}
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
