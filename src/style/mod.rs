/*!
  The style for shapes and text, font, color, etc.
*/
mod color;
mod font;
mod palette;
use std::borrow::Borrow;

pub use color::{
    Black, Blue, Color, Cyan, Green, HSLColor, Magenta, PaletteColor, RGBAColor, RGBColor, Red,
    SimpleColor, Transparent, White, Yellow,
};

pub use font::{FontDesc, FontError, FontResult, FontTransform, IntoFont, LayoutBox};
pub use palette::*;

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: &'a FontDesc<'a>,
    pub color: RGBAColor,
}

impl<'a> TextStyle<'a> {
    /// Determine the color of the style
    pub fn color<C: Color>(&self, color: &'a C) -> Self {
        Self {
            font: self.font,
            color: color.to_rgba(),
        }
    }
}

/// Make sure that we are able to automatically copy the `TextStyle`
impl<'a, 'b: 'a> Into<TextStyle<'a>> for &'b TextStyle<'a> {
    fn into(self) -> TextStyle<'a> {
        self.clone()
    }
}

impl<'a, T: Borrow<FontDesc<'a>>> From<&'a T> for TextStyle<'a> {
    fn from(font: &'a T) -> Self {
        Self {
            font: font.borrow(),
            color: Black.to_rgba(),
        }
    }
}

/// Style for any of shape
#[derive(Clone)]
pub struct ShapeStyle {
    pub color: RGBAColor,
    pub filled: bool,
}

impl ShapeStyle {
    /// Make a filled shape style
    pub fn filled(&self) -> Self {
        Self {
            color: self.color.to_rgba(),
            filled: true,
        }
    }
}

impl<'a, T: Color> From<&'a T> for ShapeStyle {
    fn from(f: &'a T) -> Self {
        ShapeStyle {
            color: f.to_rgba(),
            filled: false,
        }
    }
}
