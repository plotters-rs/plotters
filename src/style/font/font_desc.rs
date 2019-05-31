use super::{FontData, FontDataInternal};
use crate::style::{Color, TextStyle};

use std::convert::From;

/// The error type for the font implementation
pub type FontError = <FontDataInternal as FontData>::ErrorType;

/// The type we used to represent a result of any font operations
pub type FontResult<T> = Result<T, FontError>;

/// Describes a font
pub struct FontDesc<'a> {
    size: f64,
    name: &'a str,
    data: FontResult<FontDataInternal>,
}

impl<'a> From<&'a str> for FontDesc<'a> {
    fn from(from: &'a str) -> FontDesc<'a> {
        FontDesc::new(from, 1.0)
    }
}

impl<'a, T: Into<f64>> From<(&'a str, T)> for FontDesc<'a> {
    fn from((typeface, size): (&'a str, T)) -> FontDesc<'a> {
        FontDesc::new(typeface, size.into())
    }
}

pub trait IntoFont<'a> {
    fn into_font(self) -> FontDesc<'a>;
}

impl<'a, T: Into<FontDesc<'a>>> IntoFont<'a> for T {
    fn into_font(self) -> FontDesc<'a> {
        self.into()
    }
}

impl<'a> FontDesc<'a> {
    /// Create a new font
    pub fn new(typeface: &'a str, size: f64) -> Self {
        Self {
            size,
            name: typeface,
            data: FontDataInternal::new(typeface),
        }
    }

    /// Create a new font desc with the same font but different size
    pub fn resize(&self, size: f64) -> FontDesc<'a> {
        Self {
            size,
            name: self.name,
            data: self.data.clone(),
        }
    }

    /// Set the color of the font and return the result text style object
    pub fn color<C: Color>(&'a self, color: &'a C) -> TextStyle<'a> {
        TextStyle { font: self, color }
    }

    /// Get the name of the font
    pub fn get_name(&self) -> &'a str {
        self.name
    }

    /// Get the size of font
    pub fn get_size(&self) -> f64 {
        self.size
    }

    /// Get the size of the text if rendered in this font
    pub fn layout_box(&self, text: &str) -> FontResult<((i32, i32), (i32, i32))> {
        match &self.data {
            Ok(ref font) => font.estimate_layout(self.size, text),
            Err(e) => Err(e.clone()),
        }
    }

    /// Get the size of the text if rendered in this font
    pub fn box_size(&self, text: &str) -> FontResult<(u32, u32)> {
        let ((min_x, min_y), (max_x, max_y)) = self.layout_box(text)?;
        Ok(((max_x - min_x) as u32, (max_y - min_y) as u32))
    }

    /// Actually draws a font with a drawing function
    pub fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        text: &str,
        (x, y): (i32, i32),
        draw: DrawFunc,
    ) -> FontResult<Result<(), E>> {
        match &self.data {
            Ok(ref font) => font.draw((x, y), self.size, text, draw),
            Err(e) => Err(e.clone()),
        }
    }
}
