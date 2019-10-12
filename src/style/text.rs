use super::color::{Color, RGBAColor};
use super::font::{FontDesc, FontTransform};
use super::size::{HasDimension, SizeDesc};
use super::BLACK;

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: FontDesc<'a>,
    pub color: RGBAColor,
}

pub trait IntoTextStyle<'a> {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a>;
}

impl<'a> IntoTextStyle<'a> for FontDesc<'a> {
    fn into_text_style<P: HasDimension>(self, _: &P) -> TextStyle<'a> {
        self.into()
    }
}

impl<'a> IntoTextStyle<'a> for TextStyle<'a> {
    fn into_text_style<P: HasDimension>(self, _: &P) -> TextStyle<'a> {
        self
    }
}

impl<'a, T: SizeDesc> IntoTextStyle<'a> for (&'a str, T) {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a> {
        (self.0, self.1.in_pixels(parent)).into()
    }
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
