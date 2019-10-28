use super::{FontData, FontDataInternal};
use crate::style::{Color, LayoutBox, TextStyle};

use std::convert::From;

/// The error type for the font implementation
pub type FontError = <FontDataInternal as FontData>::ErrorType;

/// The type we used to represent a result of any font operations
pub type FontResult<T> = Result<T, FontError>;

/// Specifying text transformations
#[derive(Clone)]
pub enum FontTransform {
    /// Nothing to transform
    None,
    /// Rotating the text 90 degree clockwise
    Rotate90,
    /// Rotating the text 180 degree clockwise
    Rotate180,
    /// Rotating the text 270 degree clockwise
    Rotate270,
}

impl FontTransform {
    /// Compute the offset of the "top-left" corner of the text.
    /// "Top-left" defined as the first char's top-left in reading orientation.
    ///
    /// - `layout`: The bouncing box of the text
    /// - **returns**: The offset in pixels
    pub fn offset(&self, layout: LayoutBox) -> (i32, i32) {
        match self {
            FontTransform::None => (0, 0),
            FontTransform::Rotate90 => ((layout.1).1 - (layout.0).1, 0),
            FontTransform::Rotate180 => ((layout.1).0 - (layout.0).0, (layout.1).1 - (layout.0).1),
            FontTransform::Rotate270 => (0, (layout.1).0 - (layout.0).0),
        }
    }

    /// Transform the coordinate to perform the rotation
    ///
    /// - `x`: The x coordinate in pixels before transform
    /// - `y`: The y coordinate in pixels before transform
    /// - **returns**: The coordinate after transform
    pub fn transform(&self, x: i32, y: i32) -> (i32, i32) {
        match self {
            FontTransform::None => (x, y),
            FontTransform::Rotate90 => (-y, x),
            FontTransform::Rotate180 => (-x, -y),
            FontTransform::Rotate270 => (y, -x),
        }
    }
}

/// Describes a font
#[derive(Clone)]
pub struct FontDesc<'a> {
    size: f64,
    family: FontFamily<'a>,
    data: FontResult<FontDataInternal>,
    transform: FontTransform,
    style: FontStyle,
}

/// Describes font family.
/// This can be either a specific font family name, such as "arial",
/// or a general font family class, such as "serif" and "sans-serif"
#[derive(Clone, Copy)]
pub enum FontFamily<'a> {
    /// The system default serif font family
    Serif,
    /// The system default sans-serif font family
    SansSerif,
    /// The system default monospace font
    Monospace,
    /// A specific font family name
    Name(&'a str),
}

impl<'a> FontFamily<'a> {
    /// Make a CSS compatible string for the font family name.
    /// This can be used as the value of `font-family` attribute in SVG.
    pub fn as_str(&self) -> &str {
        match self {
            FontFamily::Serif => "serif",
            FontFamily::SansSerif => "sans-serif",
            FontFamily::Monospace => "monospace",
            FontFamily::Name(face) => face,
        }
    }
}

impl<'a> From<&'a str> for FontFamily<'a> {
    fn from(from: &'a str) -> FontFamily<'a> {
        match from.to_lowercase().as_str() {
            "serif" => FontFamily::Serif,
            "sans-serif" => FontFamily::SansSerif,
            "monospace" => FontFamily::Monospace,
            _ => FontFamily::Name(from),
        }
    }
}

/// Describes the font style. Such as Italic, Oblique, etc.
#[derive(Clone, Copy)]
pub enum FontStyle {
    /// The normal style
    Normal,
    /// The oblique style
    Oblique,
    /// The italic style
    Italic,
    /// The bold style
    Bold,
}

impl FontStyle {
    /// Convert the font style into a CSS compatible string which can be used in `font-style` attribute.
    pub fn as_str(&self) -> &str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique => "oblique",
            FontStyle::Bold => "bold",
        }
    }
}

impl<'a> From<&'a str> for FontStyle {
    fn from(from: &'a str) -> FontStyle {
        match from.to_lowercase().as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            "bold" => FontStyle::Bold,
            _ => FontStyle::Normal,
        }
    }
}

impl<'a> From<&'a str> for FontDesc<'a> {
    fn from(from: &'a str) -> FontDesc<'a> {
        FontDesc::new(from.into(), 1.0, FontStyle::Normal)
    }
}

impl<'a> From<FontFamily<'a>> for FontDesc<'a> {
    fn from(family: FontFamily<'a>) -> FontDesc<'a> {
        FontDesc::new(family, 1.0, FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>> From<(FontFamily<'a>, T)> for FontDesc<'a> {
    fn from((family, size): (FontFamily<'a>, T)) -> FontDesc<'a> {
        FontDesc::new(family, size.into(), FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>> From<(&'a str, T)> for FontDesc<'a> {
    fn from((typeface, size): (&'a str, T)) -> FontDesc<'a> {
        FontDesc::new(typeface.into(), size.into(), FontStyle::Normal)
    }
}

impl<'a, T: Into<f64>, S: Into<FontStyle>> From<(FontFamily<'a>, T, S)> for FontDesc<'a> {
    fn from((family, size, style): (FontFamily<'a>, T, S)) -> FontDesc<'a> {
        FontDesc::new(family, size.into(), style.into())
    }
}

impl<'a, T: Into<f64>, S: Into<FontStyle>> From<(&'a str, T, S)> for FontDesc<'a> {
    fn from((typeface, size, style): (&'a str, T, S)) -> FontDesc<'a> {
        FontDesc::new(typeface.into(), size.into(), style.into())
    }
}

/// The trait that allows some type turns into a font description
pub trait IntoFont<'a> {
    /// Make the font description from the source type
    fn into_font(self) -> FontDesc<'a>;
}

impl<'a, T: Into<FontDesc<'a>>> IntoFont<'a> for T {
    fn into_font(self) -> FontDesc<'a> {
        self.into()
    }
}

impl<'a> FontDesc<'a> {
    /// Create a new font
    ///
    /// - `family`: The font family name
    /// - `size`: The size of the font
    /// - `style`: The font variations
    /// - **returns** The newly created font description
    pub fn new(family: FontFamily<'a>, size: f64, style: FontStyle) -> Self {
        Self {
            size,
            family,
            data: FontDataInternal::new(family, style),
            transform: FontTransform::None,
            style,
        }
    }

    /// Create a new font desc with the same font but different size
    ///
    /// - `size`: The new size to set
    /// - **returns** The newly created font descriptor with a new size
    pub fn resize(&self, size: f64) -> FontDesc<'a> {
        Self {
            size,
            family: self.family,
            data: self.data.clone(),
            transform: self.transform.clone(),
            style: self.style,
        }
    }

    /// Set the style of the font
    ///
    /// - `style`: The new style
    /// - **returns** The new font description with this style applied
    pub fn style(&self, style: FontStyle) -> Self {
        Self {
            size: self.size,
            family: self.family,
            data: self.data.clone(),
            transform: self.transform.clone(),
            style,
        }
    }

    /// Set the font transformation
    ///
    /// - `trans`: The new transformation
    /// - **returns** The new font description with this font transformation applied
    pub fn transform(&self, trans: FontTransform) -> Self {
        Self {
            size: self.size,
            family: self.family,
            data: self.data.clone(),
            transform: trans,
            style: self.style,
        }
    }

    /// Get the font transformation description
    pub fn get_transform(&self) -> FontTransform {
        self.transform.clone()
    }

    /// Set the color of the font and return the result text style object
    pub fn color<C: Color>(&self, color: &C) -> TextStyle<'a> {
        TextStyle {
            font: self.clone(),
            color: color.to_rgba(),
        }
    }

    /// Get the name of the font
    pub fn get_name(&self) -> &str {
        self.family.as_str()
    }

    /// Get the name of the style
    pub fn get_style(&self) -> FontStyle {
        self.style
    }

    /// Get the size of font
    pub fn get_size(&self) -> f64 {
        self.size
    }

    /// Get the size of the text if rendered in this font
    ///
    /// For a TTF type, zero point of the layout box is the left most baseline char of the string
    /// Thus the upper bound of the box is most likely be negative
    pub fn layout_box(&self, text: &str) -> FontResult<((i32, i32), (i32, i32))> {
        match &self.data {
            Ok(ref font) => font.estimate_layout(self.size, text),
            Err(e) => Err(e.clone()),
        }
    }

    /// Get the size of the text if rendered in this font.
    /// This is similar to `layout_box` function, but it apply the font transformation
    /// and estimate the overall size of the font
    pub fn box_size(&self, text: &str) -> FontResult<(u32, u32)> {
        let ((min_x, min_y), (max_x, max_y)) = self.layout_box(text)?;
        let (w, h) = self.get_transform().transform(max_x - min_x, max_y - min_y);
        Ok((w.abs() as u32, h.abs() as u32))
    }

    /// Actually draws a font with a drawing function
    pub fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        text: &str,
        (x, y): (i32, i32),
        draw: DrawFunc,
    ) -> FontResult<Result<(), E>> {
        match &self.data {
            Ok(ref font) => font.draw((x, y), self.size, text, self.get_transform(), draw),
            Err(e) => Err(e.clone()),
        }
    }
}
