use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::{ShapeStyle, TextStyle};

pub struct Pixel<'a, Coord> {
    pos: Coord,
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Pixel<'a, Coord> {
    pub fn new<P: Into<Coord>, S: Into<ShapeStyle<'a>>>(pos: P, style: S) -> Self {
        return Self {
            pos: pos.into(),
            style: style.into(),
        };
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Pixel<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        return std::iter::once(&self.pos);
    }
}

impl<'a, Coord: 'a> Drawable for Pixel<'a, Coord> {
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.draw_pixel((x, y), &Box::new(self.style.color));
        }
        return Ok(());
    }
}

/// An element of a series of connected lines
pub struct Path<'a, Coord> {
    points: Vec<Coord>,
    style: ShapeStyle<'a>,
}
impl<'a, Coord> Path<'a, Coord> {
    /// Create a new path
    /// - `points`: The iterator of the points
    /// - `style`: The shape style
    /// - returns the created element
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

/// A rectangle element
pub struct Rectangle<'a, Coord> {
    points: [Coord; 2],
    style: ShapeStyle<'a>,
    margin: (u32, u32, u32, u32),
}

impl<'a, Coord> Rectangle<'a, Coord> {
    /// Create a new path
    /// - `points`: The left upper and right lower coner of the rectangle
    /// - `style`: The shape style
    /// - returns the created element
    pub fn new<S: Into<ShapeStyle<'a>>>(points: [Coord; 2], style: S) -> Self {
        return Self {
            points,
            style: style.into(),
            margin: (0, 0, 0, 0),
        };
    }

    /// Set the margin of the rectangle
    /// - `t`: The top margin
    /// - `b`: The bottom margin
    /// - `l`: The left margin
    /// - `r`: The right margin
    pub fn set_margin(&mut self, t: u32, b: u32, l: u32, r: u32) -> &mut Self {
        self.margin = (t, b, l, r);
        return self;
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
            (Some(mut a), Some(mut b)) => {
                a.1 += self.margin.0 as i32;
                b.1 -= self.margin.1 as i32;
                a.0 += self.margin.2 as i32;
                b.0 -= self.margin.3 as i32;
                return backend.draw_rect(a, b, &Box::new(self.style.color), self.style.filled);
            }
            _ => {
                return Ok(());
            }
        }
    }
}

/// A text element
pub struct Text<'a, Coord> {
    text: &'a str,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord> Text<'a, Coord> {
    /// Create a new text element
    /// - `text`: The text for the element
    /// - `points`: The upper left conner for the text element
    /// - `style`: The text style
    /// - Return the newly created text element
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

/// A circle element
pub struct Circle<'a, Coord> {
    center: Coord,
    size: u32,
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Circle<'a, Coord> {
    /// Create a new circle element
    /// - `coord` The center of the circle
    /// - `size` The radius of the circle
    /// - `style` The style of the circle
    /// - Return: The newly created circle element
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

/// A text element. This is similar to the text element, but it owns the
/// string.
pub struct OwnedText<'a, Coord> {
    text: String,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord> OwnedText<'a, Coord> {
    /// Create a new owned text element
    /// - `text`: The text to create
    /// - `points`: The left upper conner
    /// - `style`: The font style
    /// - Return the newly created owned text object
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
