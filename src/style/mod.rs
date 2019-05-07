mod color;
mod font;
mod palette;
use std::borrow::Borrow;

pub use color::{
    Black, Blue, Color, Cyan, Green, Magenta, Mixable, PaletteColor, RGBColor, Red, SimpleColor,
    Transparent, White, Yellow,
};
pub use font::{FontDesc, FontError, FontResult};
pub use palette::*;

/// The object that describe the style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: &'a FontDesc<'a>,
    pub color: &'a dyn Color,
}

impl<'a> TextStyle<'a> {
    /// Make a filled shape style
    pub fn color<C: Color>(&self, color: &'a C) -> Self {
        return Self {
            font: self.font,
            color,
        };
    }
}

impl<'a, T: Borrow<FontDesc<'a>>> From<&'a T> for TextStyle<'a> {
    fn from(font: &'a T) -> Self {
        return Self {
            font: font.borrow(),
            color: &RGBColor(0, 0, 0),
        };
    }
}

/// The object that describes the style of a shape
#[derive(Clone)]
pub struct ShapeStyle<'a> {
    pub color: &'a dyn Color,
    pub filled: bool,
}

impl<'a> ShapeStyle<'a> {
    /// Make a filled shape style
    pub fn filled(&self) -> Self {
        return Self {
            color: self.color,
            filled: true,
        };
    }
}

impl<'a, T: Color> From<&'a T> for ShapeStyle<'a> {
    fn from(f: &'a T) -> Self {
        return ShapeStyle {
            color: f,
            filled: false,
        };
    }
}
