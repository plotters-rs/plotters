// pattern: Mixed (needs refactoring)

#[path = "system_source.rs"]
mod system_source;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

use swash::{
    scale::{Render, ScaleContext, Source},
    zeno::Format,
    Charmap, FontRef, GlyphId,
};

use system_source::SystemFontData;
use ttf_parser::{Face, GlyphId as TtfGlyphId};

use super::{FontData, FontFamily, FontStyle, LayoutBox};

type FontResult<T> = Result<T, FontError>;

#[derive(Debug, Clone)]
pub enum FontError {
    LockError,
    NoSuchFont(String, String),
    FontLoadError(String),
    FaceParseError(String),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FontError::LockError => write!(fmt, "could not lock mutex"),
            FontError::NoSuchFont(family, style) => {
                write!(fmt, "no such font: {} {}", family, style)
            }
            FontError::FontLoadError(e) => write!(fmt, "font loading error: {}", e),
            FontError::FaceParseError(e) => write!(fmt, "font face parse error: {}", e),
        }
    }
}

impl std::error::Error for FontError {}

lazy_static! {
    static ref DATA_CACHE: RwLock<HashMap<String, FontResult<SystemFontData>>> =
        RwLock::new(HashMap::new());
}

thread_local! {
    static FONT_OBJECT_CACHE: RefCell<HashMap<String, FontExt>> = RefCell::new(HashMap::new());
    static SCALE_CONTEXT: RefCell<ScaleContext> = RefCell::new(ScaleContext::new());
}

const PLACEHOLDER_CHAR: char = '�';

#[derive(Clone)]
struct FontExt {
    bytes: Arc<Vec<u8>>,
    index: usize,
}

impl FontExt {
    fn new(data: SystemFontData) -> FontResult<Self> {
        FontRef::from_index(data.bytes.as_slice(), data.index).ok_or_else(|| {
            FontError::FontLoadError(format!("invalid font data at index {}", data.index))
        })?;
        Face::parse(data.bytes.as_slice(), data.index as u32)
            .map_err(|err| FontError::FaceParseError(err.to_string()))?;
        Ok(Self {
            bytes: data.bytes,
            index: data.index,
        })
    }

    fn font_ref(&self) -> FontResult<FontRef<'_>> {
        FontRef::from_index(self.bytes.as_slice(), self.index).ok_or_else(|| {
            FontError::FontLoadError(format!("invalid font data at index {}", self.index))
        })
    }

    fn cache_id(&self) -> [u64; 2] {
        [Arc::as_ptr(&self.bytes) as usize as u64, self.index as u64]
    }

    fn query_kerning_table(&self, prev: GlyphId, next: GlyphId) -> f32 {
        let Ok(face) = Face::parse(self.bytes.as_slice(), self.index as u32) else {
            return 0.0;
        };
        if let Some(kern) = face.tables().kern {
            let kern = kern
                .subtables
                .into_iter()
                .filter(|st| st.horizontal && !st.variable)
                .find_map(|st| st.glyphs_kerning(TtfGlyphId(prev), TtfGlyphId(next)))
                .unwrap_or(0);
            return kern as f32;
        }
        0.0
    }
}

/// Lazily load font data. Font type doesn't own actual data, which
/// lives in the cache.
fn load_font_data(face: FontFamily, style: FontStyle) -> FontResult<FontExt> {
    let key = cache_key(face, style);

    // First, we try to find the font object for current thread
    if let Some(font_object) = FONT_OBJECT_CACHE.with(|font_object_cache| {
        font_object_cache
            .borrow()
            .get(Borrow::<str>::borrow(&key))
            .cloned()
    }) {
        return Ok(font_object);
    }

    // Then we need to check if the data cache contains the font data
    if let Some(data) = DATA_CACHE
        .read()
        .map_err(|_| FontError::LockError)?
        .get(Borrow::<str>::borrow(&key))
        .cloned()
    {
        let font = FontExt::new(data?)?;
        cache_font_object(key, &font);
        return Ok(font);
    }

    let data = system_source::load(face, style);
    DATA_CACHE
        .write()
        .map_err(|_| FontError::LockError)?
        .insert(key.clone(), data.clone());

    let font = FontExt::new(data?)?;
    cache_font_object(key, &font);
    Ok(font)
}

fn cache_key(face: FontFamily<'_>, style: FontStyle) -> String {
    match style {
        FontStyle::Normal => face.as_str().to_owned(),
        _ => format!("{}, {}", face.as_str(), style.as_str()),
    }
}

fn cache_font_object(key: String, font: &FontExt) {
    FONT_OBJECT_CACHE.with(|font_object_cache| {
        font_object_cache.borrow_mut().insert(key, font.clone());
    });
}

fn glyph_for_char(charmap: &Charmap<'_>, c: char) -> Option<GlyphId> {
    let glyph_id = charmap.map(c);
    (glyph_id != 0).then_some(glyph_id)
}

fn scale_design_units(value: f32, em: f32, units_per_em: u16) -> f32 {
    if units_per_em == 0 {
        0.0
    } else {
        value * em / units_per_em as f32
    }
}

#[derive(Clone)]
pub struct FontDataInternal(FontExt);

impl FontData for FontDataInternal {
    type ErrorType = FontError;

    fn new(family: FontFamily, style: FontStyle) -> Result<Self, FontError> {
        Ok(FontDataInternal(load_font_data(family, style)?))
    }

    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let pixel_per_em = size / 1.24;
        let font = &self.0;
        let font_ref = font.font_ref()?;
        let metrics = font_ref.metrics(&[]);
        let glyph_metrics = font_ref.glyph_metrics(&[]).scale(pixel_per_em as f32);
        let charmap = font_ref.charmap();

        let mut x_pixels = 0f32;

        let mut prev = None;
        let place_holder = glyph_for_char(&charmap, PLACEHOLDER_CHAR);

        for c in text.chars() {
            if let Some(glyph_id) = glyph_for_char(&charmap, c).or(place_holder) {
                x_pixels += glyph_metrics.advance_width(glyph_id);
                if let Some(pc) = prev {
                    x_pixels += scale_design_units(
                        font.query_kerning_table(pc, glyph_id),
                        pixel_per_em as f32,
                        metrics.units_per_em,
                    );
                }
                prev = Some(glyph_id);
            }
        }

        Ok(((0, 0), (x_pixels as i32, pixel_per_em as i32)))
    }

    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        (base_x, mut base_y): (i32, i32),
        size: f64,
        text: &str,
        mut draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        let em = (size / 1.24) as f32;

        let mut x = base_x as f32;
        let font = &self.0;
        let font_ref = font.font_ref()?;
        let metrics = font_ref.metrics(&[]);
        let glyph_metrics = font_ref.glyph_metrics(&[]).scale(em);
        let charmap = font_ref.charmap();

        base_y -= (0.24 * em) as i32;

        let mut prev = None;
        let place_holder = glyph_for_char(&charmap, PLACEHOLDER_CHAR);

        let render_sources = [Source::Outline];

        let draw_result = SCALE_CONTEXT.with(|scale_context| {
            let mut scale_context = scale_context.borrow_mut();
            let mut scaler = scale_context
                .builder_with_id(font_ref, font.cache_id())
                .size(em)
                .hint(true)
                .build();
            let mut renderer = Render::new(&render_sources);
            renderer.format(Format::Alpha);

            for c in text.chars() {
                if let Some(glyph_id) = glyph_for_char(&charmap, c).or(place_holder) {
                    if let Some(pc) = prev {
                        x += scale_design_units(
                            font.query_kerning_table(pc, glyph_id),
                            em,
                            metrics.units_per_em,
                        );
                    }

                    let base_x = x as i32;

                    if let Some(image) = renderer.render(&mut scaler, glyph_id) {
                        let width = image.placement.width as usize;
                        let height = image.placement.height as usize;

                        for dy in 0..height {
                            for dx in 0..width {
                                let alpha = image.data[dy * width + dx] as f32 / 255.0;
                                if let Err(e) = draw(
                                    base_x + image.placement.left + dx as i32,
                                    base_y - image.placement.top + dy as i32,
                                    alpha,
                                ) {
                                    return Err(e);
                                }
                            }
                        }
                    }

                    x += glyph_metrics.advance_width(glyph_id);

                    prev = Some(glyph_id);
                }
            }
            Ok(())
        });
        Ok(draw_result)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_font_cache() -> FontResult<()> {
        // We cannot only check the size of font cache, because
        // the test case may be run in parallel. Thus the font cache
        // may contains other fonts.
        let _a = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        assert!(DATA_CACHE.read().unwrap().contains_key("serif"));

        let _b = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        assert!(DATA_CACHE.read().unwrap().contains_key("serif"));

        // TODO: Check they are the same

        Ok(())
    }

    #[test]
    fn draw_glyphs_stay_in_expected_bounds() -> FontResult<()> {
        let cases = [
            (FontFamily::SansSerif, FontStyle::Normal),
            (FontFamily::Serif, FontStyle::Bold),
        ];

        for (family, style) in cases {
            assert_draw_sanity(family, style)?;
        }

        Ok(())
    }

    fn assert_draw_sanity(family: FontFamily<'_>, style: FontStyle) -> FontResult<()> {
        let size = 32.0;
        let em = size / 1.24;
        let font = FontDataInternal::new(family, style)?;
        let mut samples = Vec::new();

        let draw_result = font.draw((0, size as i32), size, "Hg", |x, y, alpha| {
            samples.push((x, y, alpha));
            Ok::<(), ()>(())
        })?;
        assert!(draw_result.is_ok());

        assert!(
            samples.iter().any(|(_, _, alpha)| *alpha > 0.8),
            "expected at least one high-alpha glyph sample"
        );
        assert!(
            samples
                .iter()
                .all(|(_, _, alpha)| alpha.is_finite() && (0.0..=1.0).contains(alpha)),
            "all alpha samples should be finite and normalized"
        );

        let touched: Vec<_> = samples
            .iter()
            .filter(|(_, _, alpha)| *alpha > 0.0)
            .collect();
        assert!(!touched.is_empty(), "expected non-empty touched bounds");

        let min_x = touched.iter().map(|(x, _, _)| *x).min().unwrap();
        let max_x = touched.iter().map(|(x, _, _)| *x).max().unwrap();
        let min_y = touched.iter().map(|(_, y, _)| *y).min().unwrap();
        let max_y = touched.iter().map(|(_, y, _)| *y).max().unwrap();

        assert!(min_x >= 0, "glyphs drifted left: min_x={}", min_x);
        assert!(min_y >= 0, "glyphs drifted above canvas: min_y={}", min_y);
        assert!(
            max_x <= (3.0 * em) as i32,
            "glyphs drifted right: max_x={}",
            max_x
        );
        assert!(
            max_y <= (1.5 * em) as i32,
            "glyphs drifted below canvas: max_y={}",
            max_y
        );

        Ok(())
    }
}
