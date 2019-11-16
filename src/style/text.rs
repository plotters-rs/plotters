use super::color::{Color, RGBAColor};
use super::font::{FontDesc, FontFamily, FontStyle, FontTransform};
use super::size::{HasDimension, SizeDesc};
use super::BLACK;

/// The alignment of the text.
///
/// Used to determine the invariant (anchor) points for backends
/// which render the text on the client side.
#[derive(Copy, Clone)]
pub enum TextAlignment {
    /// Left alignment
    Left,
    /// Right alignment
    Right,
    /// Center alignment
    Center,
}

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: FontDesc<'a>,
    pub color: RGBAColor,
    pub alignment: TextAlignment,
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

impl<'a> IntoTextStyle<'a> for FontFamily<'a> {
    fn into_text_style<P: HasDimension>(self, _: &P) -> TextStyle<'a> {
        self.into()
    }
}

impl<'a, T: SizeDesc> IntoTextStyle<'a> for (&'a str, T) {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a> {
        (self.0, self.1.in_pixels(parent)).into()
    }
}

impl<'a, T: SizeDesc> IntoTextStyle<'a> for (FontFamily<'a>, T) {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a> {
        (self.0, self.1.in_pixels(parent)).into()
    }
}

impl<'a, T: SizeDesc> IntoTextStyle<'a> for (&'a str, T, FontStyle) {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a> {
        Into::<FontDesc>::into((self.0, self.1.in_pixels(parent), self.2)).into()
    }
}

impl<'a, T: SizeDesc> IntoTextStyle<'a> for (FontFamily<'a>, T, FontStyle) {
    fn into_text_style<P: HasDimension>(self, parent: &P) -> TextStyle<'a> {
        Into::<FontDesc>::into((self.0, self.1.in_pixels(parent), self.2)).into()
    }
}

impl<'a> TextStyle<'a> {
    /// Sets the color of the style.
    ///
    /// - `color`: The required color
    /// - **returns** The up-to-dated text style
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let style = TextStyle::from(("sans-serif", 20).into_font()).color(&RED);
    /// ```
    pub fn color<C: Color>(&self, color: &'a C) -> Self {
        Self {
            font: self.font.clone(),
            color: color.to_rgba(),
            alignment: self.alignment,
        }
    }

    /// Sets the font transformation of the style.
    ///
    /// - `trans`: The required transformation
    /// - **returns** The up-to-dated text style
    ///
    /// ```rust
    /// use plotters::prelude::*;
    ///
    /// let style = TextStyle::from(("sans-serif", 20).into_font()).transform(FontTransform::Rotate90);
    /// ```
    pub fn transform(&self, trans: FontTransform) -> Self {
        Self {
            font: self.font.clone().transform(trans),
            color: self.color.clone(),
            alignment: self.alignment,
        }
    }

    /// Sets the text alignment of the style.
    ///
    /// - `color`: The required alignment
    /// - **returns** The up-to-dated text style
    ///
    /// ```rust
    /// use plotters::prelude::*;
    /// use plotters::style::TextAlignment;
    ///
    /// let style = TextStyle::from(("sans-serif", 20).into_font()).alignment(TextAlignment::Right);
    /// ```
    pub fn alignment(&self, alignment: TextAlignment) -> Self {
        Self {
            font: self.font.clone(),
            color: self.color.clone(),
            alignment,
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
            alignment: TextAlignment::Left,
        }
    }
}
