use super::LayoutBox;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Parses font bytes into a backend-specific font object.
pub trait FontEngine: Send + Sync {
    fn parse(&self, data: Arc<[u8]>, index: u32) -> Result<Arc<dyn ParsedFont>, FontError>;
}

/// A parsed font that can shape text and rasterize glyph masks.
pub trait ParsedFont: Send + Sync {
    fn shape(&self, text: &str, font_size_px: f32) -> Result<ShapedRun, FontError>;
    /// Rasterize a single glyph at the requested pixel size.
    ///
    /// `subpixel_offset` carries the fractional offset of the glyph origin within
    /// its target pixel cell, in `[0, 1)`. Implementations should fold it into
    /// the rasterization so strokes that fall between integer pixel columns are
    /// anti-aliased correctly instead of being rounded away.
    fn rasterize(
        &self,
        glyph_id: u32,
        font_size_px: f32,
        subpixel_offset: Vector2F,
    ) -> Result<CoverageMask, FontError>;
}

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
pub struct ShapedRun {
    pub glyphs: Vec<PositionedGlyph>,
    pub bounds: LayoutBox,
}

/// A glyph positioned relative to the run origin.
pub struct PositionedGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// A dense grayscale coverage mask.
pub struct CoverageMask {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
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
    /// Internal font state could not be locked.
    LockError,
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
            FontError::LockError => write!(fmt, "failed to lock font state"),
        }
    }
}

impl Error for FontError {}
