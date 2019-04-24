mod color;
mod font;
mod plattle;

pub use color::{Color, SimpleColor, RGBColor, PlattleColor, Mixable};
pub use plattle::*;
pub use font::{FontDesc, FontResult, FontError};

/// The object that describe the style of a text
#[derive(Clone)]
pub struct TextStyle<'a> {
    pub font: &'a FontDesc<'a>,
    pub color: &'a dyn Color,
}

/// The object that describes the style of a shape
#[derive(Clone)]
pub struct ShapeStyle<'a> {
    pub color: &'a dyn Color,
}
