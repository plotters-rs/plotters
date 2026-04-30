use super::context::{registered_font, RegisteredFont};
use super::engine::FontEngine;
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
    _priv: (),
}

impl fmt::Display for InvalidFont {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "invalid font data")
    }
}

impl Error for InvalidFont {}

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
        .map_err(|_| InvalidFont { _priv: () })?;

    REGISTERED_FONTS
        .lock()
        .map_err(|_| InvalidFont { _priv: () })?
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
