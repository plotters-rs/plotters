use super::LayoutBox;
use std::error::Error;
use std::fmt;

/// A two-dimensional vector in floating-point pixel coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vector2F {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vector2F {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A shaped single-line run.
pub(crate) struct ShapedRun {
    pub(crate) glyphs: Vec<PositionedGlyph>,
    pub(crate) bounds: LayoutBox,
}

/// A glyph positioned relative to the run origin.
pub(crate) struct PositionedGlyph {
    pub(crate) id: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

/// A dense grayscale coverage mask.
pub(crate) struct CoverageMask {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data: Vec<u8>,
}

/// The error type for the native font pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontError {
    /// The font bytes could not be parsed.
    InvalidFontData(String),
    /// The requested font collection index does not exist.
    InvalidFontIndex(u32),
    /// The requested font family and style are not available in the active context.
    NotInContext {
        /// The requested family name.
        family: String,
        /// The requested style name.
        style: String,
    },
    /// The request could only be satisfied by system fonts, but system lookup is disabled.
    SystemFontsDisabled {
        /// The requested family name.
        family: String,
    },
    /// A candidate font could not be loaded.
    FontUnavailable {
        /// The requested family name.
        family: String,
        /// The requested style name.
        style: String,
    },
    /// A glyph outline could not be converted into a coverage mask.
    RasterizeError(String),
}

impl fmt::Display for FontError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontError::InvalidFontData(err) => write!(fmt, "invalid font data: {}", err),
            FontError::InvalidFontIndex(index) => write!(fmt, "invalid font index: {}", index),
            FontError::NotInContext { family, style } => {
                write!(fmt, "font is not in context: {} {}", family, style)
            }
            FontError::SystemFontsDisabled { family } => {
                write!(fmt, "system fonts are disabled for family: {}", family)
            }
            FontError::FontUnavailable { family, style } => {
                write!(fmt, "font is unavailable: {} {}", family, style)
            }
            FontError::RasterizeError(err) => write!(fmt, "failed to rasterize glyph: {}", err),
        }
    }
}

impl Error for FontError {}
