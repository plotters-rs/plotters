/*!
  The style for shapes and text, font, color, etc.
*/
mod color;
pub mod colors;
mod font;
mod palette;
mod shape;
mod size;
mod text;

#[cfg(feature = "palette_ext")]
mod palette_ext;

/// Definitions of palettes of accessibility
pub use self::palette::*;
pub use color::{Color, HSLColor, PaletteColor, RGBAColor, RGBColor, SimpleColor};
pub use colors::{BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, TRANSPARENT, WHITE, YELLOW};
pub use font::{FontDesc, FontError, FontFamily, FontResult, FontTransform, IntoFont, LayoutBox};
pub use shape::ShapeStyle;
pub use size::{AsRelative, RelativeSize, SizeDesc};
pub use text::{IntoTextStyle, TextStyle};
