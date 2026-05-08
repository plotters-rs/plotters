// pattern: Mixed (needs refactoring)

use super::engine::{CoverageMask, FontEngine, FontError, ParsedFont, Vector2F};
use super::harfrust_engine::HarfrustEngine;
use super::system::SystemFontSource;
use super::LayoutBox;
use once_cell::sync::Lazy;
use plotters_backend::{FontFamily, FontStyle};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

type FontResult<T> = Result<T, FontError>;

#[cfg(feature = "ab_glyph")]
const DEFAULT_ENABLE_SYSTEM: bool = false;
#[cfg(not(feature = "ab_glyph"))]
const DEFAULT_ENABLE_SYSTEM: bool = true;

// Strong refs: parsed fonts intern for the process lifetime, so that repeated
// resolves return the same `Arc<dyn ParsedFont>` and the glyph cache keyed by
// `Arc::as_ptr` cannot suffer from heap address reuse.
static GLOBAL_PARSED: Lazy<Mutex<HashMap<FontFingerprint, Arc<dyn ParsedFont>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

thread_local! {
    static FONT_CTX_STACK: RefCell<Vec<Arc<FontContext>>> = const { RefCell::new(Vec::new()) };
}

/// Font state used while estimating and drawing text. Always handed around as
/// `Arc<FontContext>`, so the struct holds the shared state directly rather
/// than via an inner Arc.
pub(crate) struct FontContext {
    engine: Arc<dyn FontEngine>,
    system: Mutex<SystemFontSource>,
    glyphs: Mutex<HashMap<GlyphCacheKey, Arc<CoverageMask>>>,
    explicit: Vec<RegisteredFont>,
    enable_system: bool,
    include_registered: bool,
    // When true, named family lookups that miss every registered/system font
    // fall through to fontique's Latin-script fallback chain instead of
    // erroring. Only the process-wide system_default() turns this on, so
    // explicit `with_fonts(...)` contexts stay strict (asking for an
    // unregistered name is still a hard miss).
    fallback_unresolved_names: bool,
}

pub(crate) enum FontDrawError<E> {
    Font(FontError),
    Draw(E),
}

impl<E> From<FontError> for FontDrawError<E> {
    fn from(err: FontError) -> Self {
        Self::Font(err)
    }
}

#[derive(Clone)]
pub(crate) struct RegisteredFont {
    family: String,
    style: FontStyle,
    data: Arc<[u8]>,
    index: u32,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct FontFingerprint {
    hash: u64,
    len: usize,
    index: u32,
}

// Number of subpixel positions cached per axis. 4 is the FreeType default and
// the perceptual sweet spot: glyph spacing is preserved without exploding the
// cache (key space grows by 16x).
const SUBPIXEL_LEVELS: u32 = 4;

#[derive(Hash, PartialEq, Eq)]
struct GlyphCacheKey {
    font_ptr: usize,
    glyph_id: u32,
    size_bits: u32,
    sx_quantum: u8,
    sy_quantum: u8,
}

impl FontContext {
    /// Returns the process default font context.
    pub(crate) fn system_default() -> Arc<FontContext> {
        static DEFAULT: OnceLock<Arc<FontContext>> = OnceLock::new();
        DEFAULT
            .get_or_init(|| {
                let mut ctx = FontContext::new();
                // Restore fontconfig-style implicit fallback for the global
                // default context. This lets `("Calibri", ..)` on a host
                // without Calibri render via the closest Latin-script match
                // -- the same behaviour the old font-kit/`ttf` backend had
                // through fontconfig. Explicit `with_fonts(...)` contexts
                // intentionally stay strict.
                ctx.fallback_unresolved_names = true;
                #[cfg(feature = "ab_glyph")]
                let ctx = ctx.include_registered();
                Arc::new(ctx)
            })
            .clone()
    }

    /// Creates a font context with default settings.
    pub(crate) fn new() -> Self {
        Self {
            engine: Arc::new(HarfrustEngine),
            system: Mutex::new(SystemFontSource::new(DEFAULT_ENABLE_SYSTEM)),
            glyphs: Mutex::new(HashMap::new()),
            explicit: Vec::new(),
            enable_system: DEFAULT_ENABLE_SYSTEM,
            include_registered: false,
            fallback_unresolved_names: false,
        }
    }

    /// Adds a named font to this context.
    pub(crate) fn with_font(
        mut self,
        name: &str,
        style: FontStyle,
        bytes: impl Into<Arc<[u8]>>,
    ) -> Self {
        self.explicit.push(RegisteredFont {
            family: name.to_owned(),
            style,
            data: bytes.into(),
            index: 0,
        });
        self
    }

    /// Prevents this context from resolving fonts from the operating system.
    /// Used by unit tests to make resolution deterministic on hosts whose
    /// fontique enumeration would otherwise pollute the test font set.
    #[cfg(test)]
    pub(crate) fn disable_system_fonts(mut self) -> Self {
        self.enable_system = false;
        self.system = Mutex::new(SystemFontSource::new(false));
        self
    }

    /// Includes fonts registered through the legacy `register_font` API.
    #[cfg(feature = "ab_glyph")]
    pub(crate) fn include_registered(mut self) -> Self {
        self.include_registered = true;
        self
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
    ) -> Result<(), FontDrawError<E>> {
        let font = self.resolve(family, style)?;
        let run = font.shape(text, size as f32)?;

        for glyph in run.glyphs {
            let (int_x, sx_quantum) = split_subpixel(glyph.x);
            let (int_y, sy_quantum) = split_subpixel(glyph.y);
            let mask =
                self.rasterize_cached(&font, glyph.id, size as f32, sx_quantum, sy_quantum)?;
            for row in 0..mask.height {
                for col in 0..mask.width {
                    let index = (row * mask.width + col) as usize;
                    let alpha = mask.data[index] as f32 / 255.0;
                    if alpha == 0.0 {
                        continue;
                    }
                    let x = base_x + int_x + mask.left + col as i32;
                    let y = base_y + int_y + mask.top + row as i32;
                    draw(x, y, alpha).map_err(FontDrawError::Draw)?;
                }
            }
        }

        Ok(())
    }

    fn rasterize_cached(
        &self,
        font: &Arc<dyn ParsedFont>,
        glyph_id: u32,
        size_px: f32,
        sx_quantum: u8,
        sy_quantum: u8,
    ) -> FontResult<Arc<CoverageMask>> {
        let key = GlyphCacheKey {
            font_ptr: Arc::as_ptr(font) as *const () as usize,
            glyph_id,
            size_bits: size_px.to_bits(),
            sx_quantum,
            sy_quantum,
        };

        if let Some(mask) = self
            .glyphs
            .lock()
            .map_err(|_| FontError::LockError)?
            .get(&key)
            .cloned()
        {
            return Ok(mask);
        }

        let subpixel_offset = Vector2F::new(
            sx_quantum as f32 / SUBPIXEL_LEVELS as f32,
            sy_quantum as f32 / SUBPIXEL_LEVELS as f32,
        );
        let mask = Arc::new(font.rasterize(glyph_id, size_px, subpixel_offset)?);
        let mut cache = self.glyphs.lock().map_err(|_| FontError::LockError)?;
        Ok(cache.entry(key).or_insert(mask).clone())
    }

    fn resolve(&self, family: FontFamily<'_>, style: FontStyle) -> FontResult<Arc<dyn ParsedFont>> {
        let source = self.resolve_source(family, style)?;
        self.parse_cached(source.data, source.index)
    }

    fn resolve_source(
        &self,
        family: FontFamily<'_>,
        style: FontStyle,
    ) -> FontResult<RegisteredFont> {
        if let Some(font) = find_registered_font(&self.explicit, family, style) {
            return Ok(font.clone());
        }

        #[cfg(feature = "ab_glyph")]
        if self.include_registered {
            if let Some(font) = super::migration::registered_fonts()
                .and_then(|fonts| find_registered_font(&fonts, family, style).cloned())
            {
                return Ok(font);
            }
        }

        if !self.enable_system {
            if !self.explicit.is_empty() || self.include_registered {
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
            .system
            .lock()
            .map_err(|_| FontError::LockError)?
            .resolve(family, style, self.fallback_unresolved_names)
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
            .cloned()
        {
            return Ok(font);
        }

        let parsed = self.engine.parse(data, index)?;
        let mut global = GLOBAL_PARSED.lock().map_err(|_| FontError::LockError)?;
        Ok(global.entry(fingerprint).or_insert(parsed).clone())
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
    let family_str = family.as_str();
    let style_str = style.as_str();

    let mut fallback = None;

    for font in fonts.iter().rev() {
        if font.family != family_str {
            continue;
        }
        if font.style.as_str() == style_str {
            return Some(font);
        }
        if fallback.is_none() && !matches!(style, FontStyle::Normal) && font.style.as_str() == FontStyle::Normal.as_str() {
            fallback = Some(font);
        }
    }

    fallback
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

/// Split a sub-pixel-precise coordinate into an integer pixel and a quantized
/// subpixel offset. `pos.fract() * SUBPIXEL_LEVELS` is rounded to the nearest
/// quantum; carry into the integer part is handled via `div_euclid` /
/// `rem_euclid` so negative coordinates behave too.
fn split_subpixel(pos: f32) -> (i32, u8) {
    let levels = SUBPIXEL_LEVELS as i32;
    let total = (pos * levels as f32).round() as i32;
    let int_part = total.div_euclid(levels);
    let quantum = total.rem_euclid(levels) as u8;
    (int_part, quantum)
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/SourceSansPro-Regular-Tiny.ttf");

    #[test]
    fn explicit_font_resolves_without_system_fonts() {
        let ctx = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
                .disable_system_fonts(),
        );

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
        let a = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, bytes.clone())
                .disable_system_fonts(),
        );
        let b = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, bytes)
                .disable_system_fonts(),
        );

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
        let ctx = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
                .disable_system_fonts(),
        );

        assert!(FontContext::current().is_none());
        {
            let _guard = push_font_context(ctx.clone());
            assert!(Arc::ptr_eq(&FontContext::current().unwrap(), &ctx));
        }
        assert!(FontContext::current().is_none());
    }

    #[test]
    fn glyph_cache_returns_same_arc_for_repeat_calls() {
        let ctx = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
                .disable_system_fonts(),
        );
        let font = ctx
            .resolve(FontFamily::Name("Fixture"), FontStyle::Normal)
            .unwrap();
        let glyph_id = font.shape("A", 24.0).unwrap().glyphs[0].id;

        let mask_a = ctx.rasterize_cached(&font, glyph_id, 24.0, 0, 0).unwrap();
        let mask_b = ctx.rasterize_cached(&font, glyph_id, 24.0, 0, 0).unwrap();
        assert!(Arc::ptr_eq(&mask_a, &mask_b));

        let mask_c = ctx.rasterize_cached(&font, glyph_id, 36.0, 0, 0).unwrap();
        assert!(!Arc::ptr_eq(&mask_a, &mask_c));

        // Different sub-pixel quanta should produce a distinct entry.
        let mask_d = ctx.rasterize_cached(&font, glyph_id, 24.0, 2, 0).unwrap();
        assert!(!Arc::ptr_eq(&mask_a, &mask_d));
    }

    #[test]
    fn split_subpixel_rounds_to_quantum() {
        assert_eq!(split_subpixel(0.0), (0, 0));
        assert_eq!(split_subpixel(0.25), (0, 1));
        assert_eq!(split_subpixel(0.5), (0, 2));
        assert_eq!(split_subpixel(0.75), (0, 3));
        // Rounding into the next pixel carries into the integer part.
        assert_eq!(split_subpixel(0.99), (1, 0));
        assert_eq!(split_subpixel(8.4), (8, 2));
        // Negative coordinates use Euclidean remainder so the quantum stays
        // non-negative.
        let (int_part, quantum) = split_subpixel(-0.1);
        assert_eq!((int_part, quantum), (0, 0));
        let (int_part, quantum) = split_subpixel(-0.6);
        assert_eq!((int_part, quantum), (-1, 2));
    }

    #[test]
    fn context_stack_pops_during_unwind() {
        let ctx = Arc::new(
            FontContext::new()
                .with_font("Fixture", FontStyle::Normal, Arc::<[u8]>::from(FONT_BYTES))
                .disable_system_fonts(),
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = push_font_context(ctx);
            panic!("drop guard");
        }));

        assert!(result.is_err());
        assert!(FontContext::current().is_none());
    }
}
