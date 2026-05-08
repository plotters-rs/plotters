use super::context::{registered_font, RegisteredFont};
use super::engine::{FontEngine, FontError};
use super::harfrust_engine::HarfrustEngine;
use once_cell::sync::Lazy;
use plotters_backend::FontStyle;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

static REGISTERED_FONTS: Lazy<Mutex<Vec<RegisteredFont>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Error returned when legacy font registration receives invalid font bytes.
#[derive(Debug, Clone)]
pub struct InvalidFont {
    reason: InvalidFontReason,
}

#[derive(Debug, Clone)]
enum InvalidFontReason {
    Parse(FontError),
    RegistryLock(String),
}

impl InvalidFont {
    fn parse(err: FontError) -> Self {
        Self {
            reason: InvalidFontReason::Parse(err),
        }
    }

    fn registry_lock(err: impl fmt::Display) -> Self {
        Self {
            reason: InvalidFontReason::RegistryLock(err.to_string()),
        }
    }
}

impl fmt::Display for InvalidFont {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            InvalidFontReason::Parse(err) => write!(fmt, "failed to register font: {}", err),
            InvalidFontReason::RegistryLock(err) => {
                write!(fmt, "failed to lock registered font registry: {}", err)
            }
        }
    }
}

impl Error for InvalidFont {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.reason {
            InvalidFontReason::Parse(err) => Some(err),
            InvalidFontReason::RegistryLock(_) => None,
        }
    }
}

/// Register a font in the process-global legacy registry.
///
/// The registry is only consulted by [`super::FontContext::system_default`] and by
/// contexts explicitly built with `include_registered`.
pub fn register_font(
    name: &str,
    style: FontStyle,
    bytes: &'static [u8],
) -> Result<(), InvalidFont> {
    let data = Arc::<[u8]>::from(bytes);
    HarfrustEngine
        .parse(data.clone(), 0)
        .map_err(InvalidFont::parse)?;

    REGISTERED_FONTS
        .lock()
        .map_err(InvalidFont::registry_lock)?
        .push(registered_font(name, style, data));
    Ok(())
}

pub(crate) fn registered_fonts() -> Option<Vec<RegisteredFont>> {
    REGISTERED_FONTS.lock().ok().map(|fonts| fonts.clone())
}

#[cfg(test)]
pub(crate) fn _reset_registry_for_tests() {
    if let Ok(mut fonts) = REGISTERED_FONTS.lock() {
        fonts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_font_preserves_parse_error() {
        let err = register_font("invalid", FontStyle::Normal, b"not a font").unwrap_err();

        assert!(err.to_string().contains("failed to register font"));
        assert!(err.source().is_some());
    }
}
