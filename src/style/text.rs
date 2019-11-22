use super::color::{Color, RGBAColor};
use super::font::{FontDesc, FontFamily, FontStyle, FontTransform};
use super::size::{HasDimension, SizeDesc};
use super::BLACK;

/// Text anchor attributes are used to properly position the text.
///
/// # Examples
///
/// In the example below, the text anchor (X) position is `Pos::new(HPos::Right, VPos::Center)`.
/// ```text
///    ***** X
/// ```
/// The position is always relative to the text regardless of its rotation.
/// In the example below, the text has style
/// `style.transform(FontTransform::Rotate90).pos(Pos::new(HPos::Center, VPos::Top))`.
/// ```text
///        *
///        *
///        * X
///        *
///        *
/// ```
pub mod text_anchor {
    /// The horizontal position of the anchor point relative to the text.
    #[derive(Clone, Copy)]
    pub enum HPos {
        /// Anchor point is on the left side of the text
        Left,
        /// Anchor point is on the right side of the text
        Right,
        /// Anchor point is in the horizontal center of the text
        Center,
    }

    /// The vertical position of the anchor point relative to the text.
    #[derive(Clone, Copy)]
    pub enum VPos {
        /// Anchor point is on the top of the text
        Top,
        /// Anchor point is in the vertical center of the text
        Center,
        /// Anchor point is on the bottom of the text
        Bottom,
    }

    /// The text anchor position.
    #[derive(Clone, Copy)]
    pub struct Pos {
        /// The horizontal position of the anchor point
        pub h_pos: HPos,
        /// The vertical position of the anchor point
        pub v_pos: VPos,
    }

    impl Pos {
        /// Create a new text anchor position.
        ///
        /// - `h_pos`: The horizontal position of the anchor point
        /// - `v_pos`: The vertical position of the anchor point
        /// - **returns** The newly created text anchor position
        ///
        /// ```rust
        /// use plotters::style::text_anchor::{Pos, HPos, VPos};
        ///
        /// let pos = Pos::new(HPos::Left, VPos::Top);
        /// ```
        pub fn new(h_pos: HPos, v_pos: VPos) -> Self {
            Pos { h_pos, v_pos }
        }

        /// Create a default text anchor position (top left).
        ///
        /// - **returns** The default text anchor position
        ///
        /// ```rust
        /// use plotters::style::text_anchor::{Pos, HPos, VPos};
        ///
        /// let pos = Pos::default();
        /// ```
        pub fn default() -> Self {
            Pos {
                h_pos: HPos::Left,
                v_pos: VPos::Top,
            }
        }
    }
}

/// Style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    /// The font description
    pub font: FontDesc<'a>,
    /// The text color
    pub color: RGBAColor,
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
            color: color.to_rgba(),
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
            color: self.color.clone(),
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
            color: self.color.clone(),
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
            color: BLACK.to_rgba(),
            pos: text_anchor::Pos::default(),
        }
    }
}
