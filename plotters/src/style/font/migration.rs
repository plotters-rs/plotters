use super::context::{registered_font, RegisteredFont};
use super::engine::FontError;
use super::harfrust_engine;
use plotters_backend::FontStyle;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

static REGISTERED_FONTS: LazyLock<Mutex<Vec<RegisteredFont>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

// Bumped on every registry mutation so per-context resolution memos know to
// re-resolve instead of serving fonts cached before the registration.
static REGISTRY_GENERATION: AtomicU64 = AtomicU64::new(0);

pub(crate) fn registry_generation() -> u64 {
    REGISTRY_GENERATION.load(Ordering::Acquire)
}

/// Error returned when legacy font registration receives invalid font bytes.
#[derive(Debug, Clone)]
pub struct InvalidFont(FontError);

impl fmt::Display for InvalidFont {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "failed to register font: {}", self.0)
    }
}

impl Error for InvalidFont {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
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
    harfrust_engine::parse(data.clone(), 0).map_err(InvalidFont)?;

    REGISTERED_FONTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(registered_font(name, style, data));
    REGISTRY_GENERATION.fetch_add(1, Ordering::Release);
    Ok(())
}

pub(crate) fn registered_fonts() -> Vec<RegisteredFont> {
    REGISTERED_FONTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

#[cfg(test)]
pub(crate) fn _reset_registry_for_tests() {
    REGISTERED_FONTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
    REGISTRY_GENERATION.fetch_add(1, Ordering::Release);
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

    #[test]
    fn registration_bumps_generation() {
        static FONT_BYTES: &[u8] =
            include_bytes!("../../../tests/fixtures/SourceSansPro-Regular-Tiny.ttf");

        let before = registry_generation();
        register_font("GenerationFixture", FontStyle::Normal, FONT_BYTES).unwrap();
        assert!(registry_generation() > before);
    }
}
