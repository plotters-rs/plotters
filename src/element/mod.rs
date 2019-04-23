mod grid;

use crate::color::Color;
use crate::drawing::DrawingBackend;
use crate::font::FontDesc;
use std::iter::{once, Once};

pub use grid::{Grid, GridDirection, GridLineIter};

/// The trait that represents an element drawed on the canvas
pub trait Element<'a, Coord:'a> {
    /// The iterator for the key points of this element
    type Points: IntoIterator<Item = &'a Coord>;

    /// The function that returns the list of key point. This is used by the
    /// framework to do the coordinate mapping
    fn points(&'a self) -> Self::Points;

    /// Actually draws the element. The key points is already translated into the 
    /// image cooridnate and can be used by DC directly
    fn draw<DC:DrawingBackend, I:Iterator<Item=(u32,u32)>>(&self, pos:I, dc: &mut DC) -> Result<(), DC::ErrorType>;
}

pub struct Path<Coord, C:Color> {
    points: Vec<Coord>, 
    color: C,
}

impl <Coord, C:Color> Path<Coord, C> {
    pub fn new(points:Vec<Coord>, color:C) -> Self {
        return Self {points, color };
    }
}

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


