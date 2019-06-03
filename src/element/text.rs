use std::borrow::Borrow;

use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::TextStyle;

/// A single line text element. This can be owned or borrowed string, dependeneds on
/// `String` or `str` moved into.
pub struct Text<'a, Coord, T: Borrow<str>> {
    text: T,
    coord: Coord,
    style: TextStyle<'a>,
}

impl<'a, Coord, T: Borrow<str>> Text<'a, Coord, T> {
    /// Create a new text element
    /// - `text`: The text for the element
    /// - `points`: The upper left conner for the text element
    /// - `style`: The text style
    /// - Return the newly created text element
    pub fn new<S: Into<TextStyle<'a>>>(text: T, points: Coord, style: S) -> Self {
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
            return backend.draw_text(
                self.text.borrow(),
                self.style.font,
                a,
                &Box::new(self.style.color),
            );
        }
        Ok(())
    }
}

/// An multi-line text element. The `Text` element allows only signle line text
/// and the `MultiLineText` supports drawing multiple lines
pub struct MultiLineText<'a, Coord, T: Borrow<str>> {
    lines: Vec<T>,
    coord: Coord,
    style: TextStyle<'a>,
    line_height: f64,
}

impl<'a, Coord, T: Borrow<str>> MultiLineText<'a, Coord, T> {
    /// Create an emply multi-line text element.
    /// Lines can be append to the empty multi-line by calling `push_line` method
    ///
    /// `pos`: The upper left corner
    /// `style`: The style of the text
    pub fn new<S: Into<TextStyle<'a>>>(pos: Coord, style: S) -> Self {
        MultiLineText {
            lines: vec![],
            coord: pos,
            style: style.into(),
            line_height: 1.5,
        }
    }

    /// Push a new line into the given multi-line text
    /// `line`: The line to be pushed
    pub fn push_line<L: Into<T>>(&mut self, line: L) {
        self.lines.push(line.into());
    }

    // TODO: The layout iterator
}

// TODO: Think about how to generalize this
impl<'a, Coord> MultiLineText<'a, Coord, &'a str> {
    /// Parse a multi-line text into an multi-line element.
    ///
    /// `text`: The text that is parsed
    /// `pos`: The position of the text
    /// `style`: The style for this text
    /// `max_width`: The width of the multi-line text element, the line will break
    /// into two lines if the line is wider than the max_width. If 0 is given, do not
    /// do any line wrapping
    pub fn from_str<ST: Into<&'a str>, S: Into<TextStyle<'a>>>(
        text: ST,
        pos: Coord,
        style: S,
        max_width: u32,
    ) -> Self {
        let text = text.into();
        let mut ret = MultiLineText::new(pos, style);
        for line in text.split(|c| c == '\n') {
            if max_width == 0 {
                ret.push_line(line);
            } else {
                let mut remaining = &line[0..];

                while remaining.len() > 0 {
                    let mut width = 0;

                    let mut left = 0;
                    while left < remaining.len() {
                        let ((x0, _), (x1, _)) = ret
                            .style
                            .font
                            .layout_box(&remaining[left..(left + 1)])
                            .unwrap_or(((0, 0), (0, 0)));
                        let w = x1 - x0;
                        width += w;
                        if width > max_width as i32 {
                            break;
                        }
                        left += 1;
                    }

                    if left == 0 {
                        left = 1;
                    }

                    let part = &remaining[0..left];
                    remaining = &remaining[left..];

                    ret.push_line(part);
                }
            }
        }
        ret
    }
}

// TODO: Render the multi-line text
