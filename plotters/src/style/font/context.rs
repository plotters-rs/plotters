// pattern: Imperative Shell

use super::engine::{FontEngine, FontError, ParsedFont};
use super::harfrust_engine::HarfrustEngine;
use super::system::SystemFontSource;
use super::LayoutBox;
use once_cell::sync::Lazy;
use plotters_backend::{FontFamily, FontStyle};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock, Weak};

type FontResult<T> = Result<T, FontError>;

#[cfg(feature = "ab_glyph")]
const DEFAULT_ENABLE_SYSTEM: bool = false;
#[cfg(not(feature = "ab_glyph"))]
const DEFAULT_ENABLE_SYSTEM: bool = true;

static GLOBAL_PARSED: Lazy<Mutex<HashMap<FontFingerprint, Weak<dyn ParsedFont>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

thread_local! {
    static FONT_CTX_STACK: RefCell<Vec<Arc<FontContext>>> = RefCell::new(Vec::new());
}

/// Font state used while estimating and drawing text.
pub struct FontContext {
    inner: Arc<FontContextInner>,
}

struct FontContextInner {
    engine: Arc<dyn FontEngine>,
    system: Mutex<SystemFontSource>,
    parsed: Mutex<HashMap<FontKey, Arc<dyn ParsedFont>>>,
    explicit: Vec<RegisteredFont>,
    enable_system: bool,
    include_registered: bool,
}

/// Builder for a [`FontContext`].
pub struct FontContextBuilder {
    explicit: Vec<RegisteredFont>,
    enable_system: bool,
    include_registered: bool,
}

#[derive(Clone)]
pub(crate) struct RegisteredFont {
    family: String,
    style: FontStyle,
    data: Arc<[u8]>,
    index: u32,
}

#[derive(Hash, PartialEq, Eq)]
struct FontKey {
    family: String,
    style: String,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct FontFingerprint {
    hash: u64,
    len: usize,
    index: u32,
}

impl FontContext {
    /// Returns the process default font context.
    pub fn system_default() -> Arc<FontContext> {
        static DEFAULT: OnceLock<Arc<FontContext>> = OnceLock::new();
        DEFAULT
            .get_or_init(|| {
                let builder = FontContextBuilder::new();
                #[cfg(feature = "ab_glyph")]
                let builder = builder.include_registered();
                builder.build()
            })
            .clone()
    }

    /// Creates a font context builder.
    pub fn builder() -> FontContextBuilder {
        FontContextBuilder::new()
    }

    pub(crate) fn current() -> Option<Arc<FontContext>> {
        FONT_CTX_STACK.with(|stack| stack.borrow().last().cloned())
    }

    pub(crate) fn current_or_default() -> Arc<FontContext> {
        Self::current().unwrap_or_else(Self::system_default)
    }

    pub(crate) fn layout_box(
        &self,
        family: FontFamily<'_>,
        style: FontStyle,
        size: f64,
        text: &str,
    ) -> FontResult<LayoutBox> {
        let font = self.resolve(family, style)?;
        Ok(font.shape(text, size as f32)?.bounds)
    }

    pub(crate) fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        family: FontFamily<'_>,
        style: FontStyle,
        size: f64,
        text: &str,
        (base_x, base_y): (i32, i32),
        mut draw: DrawFunc,
    ) -> FontResult<Result<(), E>> {
        let font = self.resolve(family, style)?;
        let run = font.shape(text, size as f32)?;

        for glyph in run.glyphs {
            let mask = font.rasterize(glyph.id, size as f32)?;
            for row in 0..mask.height {
                for col in 0..mask.width {
                    let index = (row * mask.width + col) as usize;
                    let alpha = mask.data[index] as f32 / 255.0;
                    if alpha == 0.0 {
                        continue;
                    }
                    let x = base_x + (glyph.x + mask.left as f32).round() as i32 + col as i32;
                    let y = base_y + (glyph.y + mask.top as f32).round() as i32 + row as i32;
                    if let Err(err) = draw(x, y, alpha) {
                        return Ok(Err(err));
                    }
                }
            }
        }

        Ok(Ok(()))
    }

    fn resolve(&self, family: FontFamily<'_>, style: FontStyle) -> FontResult<Arc<dyn ParsedFont>> {
        let key = FontKey {
            family: family.as_str().to_owned(),
            style: style.as_str().to_owned(),
        };

        if let Some(font) = self
            .inner
            .parsed
            .lock()
            .map_err(|_| FontError::LockError)?
            .get(&key)
            .cloned()
        {
            return Ok(font);
        }

        let source = self.resolve_source(family, style)?;
        let parsed = self.parse_cached(source.data, source.index)?;
        self.inner
            .parsed
            .lock()
            .map_err(|_| FontError::LockError)?
            .insert(key, parsed.clone());
        Ok(parsed)
    }

    fn resolve_source(
        &self,
        family: FontFamily<'_>,
        style: FontStyle,
    ) -> FontResult<RegisteredFont> {
        if let Some(font) = find_registered_font(&self.inner.explicit, family, style) {
            return Ok(font.clone());
        }

        #[cfg(feature = "ab_glyph")]
        if self.inner.include_registered {
            if let Some(font) = super::migration::registered_fonts()
                .and_then(|fonts| find_registered_font(&fonts, family, style).cloned())
            {
                return Ok(font);
            }
        }

        if !self.inner.enable_system {
            if !self.inner.explicit.is_empty() || self.inner.include_registered {
                return Err(FontError::NotInContext {
                    family: family.as_str().to_owned(),
                    style: style.as_str().to_owned(),
                });
            }
            return Err(FontError::SystemFontsDisabled {
                family: family.as_str().to_owned(),
            });
        }

        let candidate = self
            .inner
            .system
            .lock()
            .map_err(|_| FontError::LockError)?
            .resolve(family, style)
            .ok_or_else(|| FontError::NotInContext {
                family: family.as_str().to_owned(),
                style: style.as_str().to_owned(),
            })?;

        Ok(RegisteredFont {
            family: family.as_str().to_owned(),
            style,
            data: candidate.data,
            index: candidate.index,
        })
    }

    fn parse_cached(&self, data: Arc<[u8]>, index: u32) -> FontResult<Arc<dyn ParsedFont>> {
        let fingerprint = fingerprint(data.as_ref(), index);
        if let Some(font) = GLOBAL_PARSED
            .lock()
            .map_err(|_| FontError::LockError)?
            .get(&fingerprint)
            .and_then(Weak::upgrade)
        {
            return Ok(font);
        }

        let parsed = self.inner.engine.parse(data, index)?;
        GLOBAL_PARSED
            .lock()
            .map_err(|_| FontError::LockError)?
            .insert(fingerprint, Arc::downgrade(&parsed));
        Ok(parsed)
    }
}

impl FontContextBuilder {
    fn new() -> Self {
        Self {
            explicit: Vec::new(),
            enable_system: DEFAULT_ENABLE_SYSTEM,
            include_registered: false,
        }
    }

    /// Adds a named font to this context.
    pub fn with_font(mut self, name: &str, style: FontStyle, bytes: impl Into<Arc<[u8]>>) -> Self {
        self.explicit.push(RegisteredFont {
            family: name.to_owned(),
            style,
            data: bytes.into(),
            index: 0,
        });
        self
    }

    /// Prevents this context from resolving fonts from the operating system.
    pub fn disable_system_fonts(mut self) -> Self {
        self.enable_system = false;
        self
    }

    /// Allows this context to resolve fonts from the operating system.
    pub fn enable_system_fonts(mut self) -> Self {
        self.enable_system = true;
        self
    }

    /// Includes fonts registered through the legacy `register_font` API.
    #[cfg(feature = "ab_glyph")]
    pub fn include_registered(mut self) -> Self {
        self.include_registered = true;
        self
    }

    /// Builds the font context.
    pub fn build(self) -> Arc<FontContext> {
        Arc::new(FontContext {
            inner: Arc::new(FontContextInner {
                engine: Arc::new(HarfrustEngine),
                system: Mutex::new(SystemFontSource::new(self.enable_system)),
                parsed: Mutex::new(HashMap::new()),
                explicit: self.explicit,
                enable_system: self.enable_system,
                include_registered: self.include_registered,
            }),
        })
    }
}

pub(crate) struct FontContextGuard;

pub(crate) fn push_font_context(ctx: Arc<FontContext>) -> FontContextGuard {
    FONT_CTX_STACK.with(|stack| stack.borrow_mut().push(ctx));
    FontContextGuard
}

impl Drop for FontContextGuard {
    fn drop(&mut self) {
        FONT_CTX_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

#[cfg(feature = "ab_glyph")]
pub(crate) fn registered_font(
    family: impl Into<String>,
    style: FontStyle,
    data: impl Into<Arc<[u8]>>,
) -> RegisteredFont {
    RegisteredFont {
        family: family.into(),
        style,
        data: data.into(),
        index: 0,
    }
}

fn find_registered_font<'a>(
    fonts: &'a [RegisteredFont],
    family: FontFamily<'_>,
    style: FontStyle,
) -> Option<&'a RegisteredFont> {
    fonts
        .iter()
        .rev()
        .find(|font| font.family == family.as_str() && font.style.as_str() == style.as_str())
        .or_else(|| {
            if matches!(style, FontStyle::Normal) {
                None
            } else {
                fonts.iter().rev().find(|font| {
                    font.family == family.as_str()
                        && font.style.as_str() == FontStyle::Normal.as_str()
                })
            }
        })
}

fn fingerprint(data: &[u8], index: u32) -> FontFingerprint {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    FontFingerprint {
        hash: hasher.finish(),
        len: data.len(),
        index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/SourceSansPro-Regular-Tiny.ttf");

    #[test]
    fn explicit_font_resolves_without_system_fonts() {
        let ctx = FontContext::builder()
            .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
            .disable_system_fonts()
            .build();

        let bounds = ctx
            .layout_box(
                FontFamily::Name("Fixture"),
                FontStyle::Normal,
                20.0,
                "Hello",
            )
            .unwrap();

        let ((min_x, min_y), (max_x, max_y)) = bounds;
        assert!(max_x > min_x);
        assert!(max_y > min_y);

        let err = ctx
            .layout_box(
                FontFamily::Name("Missing"),
                FontStyle::Normal,
                20.0,
                "Hello",
            )
            .unwrap_err();
        assert!(matches!(err, FontError::NotInContext { .. }));
    }

    #[test]
    fn global_parse_cache_shares_fonts_between_contexts() {
        let bytes = Arc::<[u8]>::from(FONT_BYTES);
        let a = FontContext::builder()
            .with_font("Fixture", FontStyle::Normal, bytes.clone())
            .disable_system_fonts()
            .build();
        let b = FontContext::builder()
            .with_font("Fixture", FontStyle::Normal, bytes)
            .disable_system_fonts()
            .build();

        let font_a = a
            .resolve(FontFamily::Name("Fixture"), FontStyle::Normal)
            .unwrap();
        let font_b = b
            .resolve(FontFamily::Name("Fixture"), FontStyle::Normal)
            .unwrap();

        assert!(Arc::ptr_eq(&font_a, &font_b));
    }

    #[test]
    fn context_stack_pops_when_guard_drops() {
        let ctx = FontContext::builder()
            .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
            .disable_system_fonts()
            .build();

        assert!(FontContext::current().is_none());
        {
            let _guard = push_font_context(ctx.clone());
            assert!(Arc::ptr_eq(&FontContext::current().unwrap(), &ctx));
        }
        assert!(FontContext::current().is_none());
    }

    #[test]
    fn context_stack_pops_during_unwind() {
        let ctx = FontContext::builder()
            .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
            .disable_system_fonts()
            .build();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = push_font_context(ctx);
            panic!("drop guard");
        }));

        assert!(result.is_err());
        assert!(FontContext::current().is_none());
    }
}
