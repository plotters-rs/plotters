/*!
  The style for shapes and text, font, color, etc.
*/
mod color;
pub mod colors;
mod font;
mod palette;
mod size;

#[cfg(feature = "palette_ext")]
mod palette_ext;

/// Definitions of palettes of accessibility
pub use self::palette::*;
pub use color::{Color, HSLColor, PaletteColor, RGBAColor, RGBColor, SimpleColor};
pub use colors::{BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, TRANSPARENT, WHITE, YELLOW};
pub use font::{
    FontDesc, FontError, FontResult, FontTransform, IntoFont, LayoutBox, RelativeFont,
    RelativeFontDesc,
};
pub use size::{AsRelativeHeight, AsRelativeWidth, RelativeSize, SizeDesc};

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: FontDesc<'a>,
    pub color: RGBAColor,
}

impl<'a> TextStyle<'a> {
    /// Determine the color of the style
    pub fn color<C: Color>(&self, color: &'a C) -> Self {
        Self {
            font: self.font.clone(),
            color: color.to_rgba(),
        }
    }

    pub fn transform(&self, trans: FontTransform) -> Self {
        Self {
            font: self.font.clone().transform(trans),
            color: self.color.clone(),
        }
    }
}

/// Make sure that we are able to automatically copy the `TextStyle`
impl<'a, 'b: 'a> Into<TextStyle<'a>> for &'b TextStyle<'a> {
    fn into(self) -> TextStyle<'a> {
        self.clone()
    }
}

impl<'a, T: Into<FontDesc<'a>>> From<T> for TextStyle<'a> {
    fn from(font: T) -> Self {
        Self {
            font: font.into(),
            color: BLACK.to_rgba(),
        }
    }
}

/// Style for any of shape
#[derive(Clone)]
pub struct ShapeStyle {
    pub color: RGBAColor,
    pub filled: bool,
    pub stroke_width: u32,
}

impl ShapeStyle {
    /// Make a filled shape style
    pub fn filled(&self) -> Self {
        Self {
            color: self.color.to_rgba(),
            filled: true,
            stroke_width: self.stroke_width,
        }
    }

    pub fn stroke_width(&self, width: u32) -> Self {
        Self {
            color: self.color.to_rgba(),
            filled: self.filled,
            stroke_width: width,
        }
    }
}

impl<'a, T: Color> From<&'a T> for ShapeStyle {
    fn from(f: &'a T) -> Self {
        ShapeStyle {
            color: f.to_rgba(),
            filled: false,
            stroke_width: 1,
        }
    }
}
