use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::{ShapeStyle, TextStyle};

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

pub struct Rectangle<'a, Coord> {
    points: [Coord; 2],
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Rectangle<'a, Coord> {
    pub fn new<S: Into<ShapeStyle<'a>>>(points: [Coord; 2], style: S) -> Self {
        return Self {
            points,
            style: style.into(),
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Rectangle<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        return &self.points;
    }
}

impl<'a, Coord: 'a> Drawable for Rectangle<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        match (points.next(), points.next()) {
            (Some(a), Some(b)) => {
                return backend.draw_rect(a, b, &Box::new(self.style.color), self.style.filled);
            }
            _ => {
                return Ok(());
            }
        }
    }
}

pub struct Text<'a, Coord> {
    text: &'a str,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord> Text<'a, Coord> {
    pub fn new<T: AsRef<str>, S: Into<TextStyle<'a>>>(
        text: &'a T,
        points: Coord,
        style: S,
    ) -> Self {
        return Self {
            text: text.as_ref(),
            coord: points,
            style: style.into(),
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Text<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return std::iter::once(&self.coord);
    }
}

impl<'a, Coord: 'a> Drawable for Text<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some(a) = points.next() {
            return backend.draw_text(self.text, self.style.font, a, &Box::new(self.style.color));
        }
        return Ok(());
    }
}

pub struct Circle<'a, Coord> {
    center: Coord,
    size: u32,
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Circle<'a, Coord> {
    pub fn new(coord: Coord, size: u32, style: ShapeStyle<'a>) -> Self {
        return Self {
            center: coord,
            size,
            style,
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Circle<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        return std::iter::once(&self.center);
    }
}

impl<'a, Coord: 'a> Drawable for Circle<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.draw_circle(
                (x, y),
                self.size,
                &Box::new(self.style.color),
                self.style.filled,
            );
        }
        return Ok(());
    }
}

pub struct OwnedText<'a, Coord> {
    text: String,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord> OwnedText<'a, Coord> {
    pub fn new<S: Into<TextStyle<'a>>>(text: String, points: Coord, style: S) -> Self {
        return Self {
            text,
            coord: points,
            style: style.into(),
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a OwnedText<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return std::iter::once(&self.coord);
    }
}

impl<'a, Coord: 'a> Drawable for OwnedText<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some(a) = points.next() {
            return backend.draw_text(&self.text, self.style.font, a, &Box::new(self.style.color));
        }
        return Ok(());
    }
}
