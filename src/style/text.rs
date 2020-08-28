use super::color::Color;
use super::font::{FontDesc, FontError, FontFamily, FontStyle, FontTransform};
use super::size::{HasDimension, SizeDesc};
use super::BLACK;
pub use plotters_backend::text_anchor;
use plotters_backend::{BackendColor, BackendCoord, BackendStyle, BackendTextStyle};

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    /// The font description
    pub font: FontDesc<'a>,
    /// The text color
    pub color: BackendColor,
    /// The anchor point position
    pub pos: text_anchor::Pos,
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
            color: color.color(),
            pos: self.pos,
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
            color: self.color,
            pos: self.pos,
        }
    }

    /// Sets the anchor position.
    ///
    /// - `pos`: The required anchor position
    /// - **returns** The up-to-dated text style
    ///
    /// ```rust
    /// use plotters::prelude::*;
    /// use plotters::style::text_anchor::{Pos, HPos, VPos};
    ///
    /// let pos = Pos::new(HPos::Left, VPos::Top);
    /// let style = TextStyle::from(("sans-serif", 20).into_font()).pos(pos);
    /// ```
    pub fn pos(&self, pos: text_anchor::Pos) -> Self {
        Self {
            font: self.font.clone(),
            color: self.color,
            pos,
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
            color: BLACK.color(),
            pos: text_anchor::Pos::default(),
        }
    }
}

impl<'a> BackendTextStyle for TextStyle<'a> {
    type FontError = FontError;
    fn color(&self) -> BackendColor {
        self.color.color()
    }

    fn size(&self) -> f64 {
        self.font.get_size()
    }

    fn transform(&self) -> FontTransform {
        self.font.get_transform()
    }

    fn style(&self) -> FontStyle {
        self.font.get_style()
    }

    #[allow(clippy::type_complexity)]
    fn layout_box(&self, text: &str) -> Result<((i32, i32), (i32, i32)), Self::FontError> {
        self.font.layout_box(text)
    }

    fn anchor(&self) -> text_anchor::Pos {
        self.pos
    }

    fn family(&self) -> FontFamily {
        self.font.get_family()
    }

    fn draw<E, DrawFunc: FnMut(i32, i32, BackendColor) -> Result<(), E>>(
        &self,
        text: &str,
        pos: BackendCoord,
        mut draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::FontError> {
        let color = self.color.color();
        self.font.draw(text, pos, move |x, y, a| {
            let mix_color = color.mix(a as f64);
            draw(x, y, mix_color)
        })
    }
}
