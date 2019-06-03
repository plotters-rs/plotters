use std::borrow::Borrow;

use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::TextStyle;

/// A text element
pub struct Text<'a, Coord, T: Borrow<str>> {
    text: T,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord, T:Borrow<str>> Text<'a, Coord, T> {
    /// Create a new text element
    /// - `text`: The text for the element
    /// - `points`: The upper left conner for the text element
    /// - `style`: The text style
    /// - Return the newly created text element
    pub fn new<S: Into<TextStyle<'a>>>(
        text: T,
        points: Coord,
        style: S,
    ) -> Self {
        Self {
            text,
            coord: points,
            style: style.into(),
        }
    }
}

impl<'b, 'a, Coord: 'a, T: Borrow<str> + 'a> PointCollection<'a, Coord> for &'a Text<'b, Coord, T> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.coord)
    }
}

impl<'a, Coord: 'a, DB: DrawingBackend, T: Borrow<str>> Drawable<DB> for Text<'a, Coord, T> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some(a) = points.next() {
            return backend.draw_text(self.text.borrow(), self.style.font, a, &Box::new(self.style.color));
        }
        Ok(())
    }
}

