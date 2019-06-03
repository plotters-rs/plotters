use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::ShapeStyle;

/// An element of a single pixel
pub struct Pixel<'a, Coord> {
    pos: Coord,
    style: ShapeStyle<'a>,
}

impl<'a, Coord> Pixel<'a, Coord> {
    pub fn new<P: Into<Coord>, S: Into<ShapeStyle<'a>>>(pos: P, style: S) -> Self {
        Self {
            pos: pos.into(),
            style: style.into(),
        }
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Pixel<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.pos)
    }
}

impl<'a, Coord: 'a, DB: DrawingBackend> Drawable<DB> for Pixel<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.draw_pixel((x, y), &Box::new(self.style.color));
        }
        Ok(())
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
        Self {
            points: points.into(),
            style: style.into(),
        }
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Path<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<'a, Coord: 'a, DB: DrawingBackend> Drawable<DB> for Path<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        backend.draw_path(points, &Box::new(self.style.color))
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
        Self {
            points,
            style: style.into(),
            margin: (0, 0, 0, 0),
        }
    }

    /// Set the margin of the rectangle
    /// - `t`: The top margin
    /// - `b`: The bottom margin
    /// - `l`: The left margin
    /// - `r`: The right margin
    pub fn set_margin(&mut self, t: u32, b: u32, l: u32, r: u32) -> &mut Self {
        self.margin = (t, b, l, r);
        self
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Rectangle<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<'a, Coord: 'a, DB: DrawingBackend> Drawable<DB> for Rectangle<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
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
                backend.draw_rect(a, b, &Box::new(self.style.color), self.style.filled)
            }
            _ => Ok(()),
        }
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
    pub fn new<S: Into<ShapeStyle<'a>>>(coord: Coord, size: u32, style: S) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'b, 'a, Coord: 'a> PointCollection<'a, Coord> for &'a Circle<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<'a, Coord: 'a, DB: DrawingBackend> Drawable<DB> for Circle<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
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
        Ok(())
    }
}
