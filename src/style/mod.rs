/*!
  The style for shapes and text, font, color, etc.
*/
mod color;
mod font;
mod palette;
use std::borrow::Borrow;

pub use color::{
    Black, Blue, Color, Cyan, Green, HSLColor, Magenta, Mixable, PaletteColor, RGBColor, Red,
    SimpleColor, Transparent, White, Yellow,
};

pub use font::{FontDesc, FontError, FontResult, FontTransform, IntoFont, LayoutBox};
pub use palette::*;

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: &'a FontDesc<'a>,
    pub color: &'a dyn Color,
}

impl<'a> TextStyle<'a> {
    /// Determine the color of the style
    pub fn color<C: Color>(&self, color: &'a C) -> Self {
        Self {
            font: self.font,
            color,
        }
    }

    // TODO: How to make the font transform inside the text style
}

impl<'a, 'b: 'a> Into<TextStyle<'a>> for &'b TextStyle<'a> {
    fn into(self) -> TextStyle<'a> {
        self.clone()
    }
}

impl<'a, T: Borrow<FontDesc<'a>>> From<&'a T> for TextStyle<'a> {
    fn from(font: &'a T) -> Self {
        Self {
            font: font.borrow(),
            color: &RGBColor(0, 0, 0),
        }
    }
}

/// Style for any of shape
#[derive(Clone)]
pub struct ShapeStyle<'a> {
    pub color: &'a dyn Color,
    pub filled: bool,
}

impl<'a> ShapeStyle<'a> {
    /// Make a filled shape style
    pub fn filled(&self) -> Self {
        Self {
            color: self.color,
            filled: true,
        }
    }
}

impl<'a, T: Color> From<&'a T> for ShapeStyle<'a> {
    fn from(f: &'a T) -> Self {
        ShapeStyle {
            color: f,
            filled: false,
        }
    }
}
