#[path = "system_source.rs"]
mod system_source;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

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

thread_local! {
    static FONT_OBJECT_CACHE: RefCell<HashMap<String, FontExt>> = RefCell::new(HashMap::new());
    static SCALE_CONTEXT: RefCell<ScaleContext> = RefCell::new(ScaleContext::new());
}

/// Substituted when the requested glyph is missing from the font.
const PLACEHOLDER_CHAR: char = '\u{FFFD}';

const RENDER_SOURCES: [Source; 1] = [Source::Outline];

#[derive(Clone)]
struct FontExt {
    bytes: Arc<Vec<u8>>,
    index: usize,
    id: u64,
}

impl FontExt {
    fn from_data(data: SystemFontData) -> Self {
        Self {
            bytes: data.bytes,
            index: data.index,
            id: data.id,
        }
    }

    fn font_ref(&self) -> FontRef<'_> {
        FontRef::from_index(self.bytes.as_slice(), self.index)
            .expect("font validated at system_source::load")
    }

    fn face(&self) -> Face<'_> {
        Face::parse(self.bytes.as_slice(), self.index as u32)
            .expect("face validated at system_source::load")
    }

    fn cache_id(&self) -> [u64; 2] {
        [self.id, self.index as u64]
    }
}

fn kerning_units(face: &Face<'_>, prev: GlyphId, next: GlyphId) -> i16 {
    let Some(kern) = face.tables().kern else {
        return 0;
    };
    kern.subtables
        .into_iter()
        .filter(|st| st.horizontal && !st.variable)
        .find_map(|st| st.glyphs_kerning(TtfGlyphId(prev), TtfGlyphId(next)))
        .unwrap_or(0)
}

/// Fetch the font for `(face, style)`, hitting the thread-local cache when
/// possible and falling back to the global byte cache in `system_source`.
fn load_font_data(face: FontFamily, style: FontStyle) -> FontResult<FontExt> {
    let key = cache_key(face, style);

    if let Some(font_object) =
        FONT_OBJECT_CACHE.with(|cache| cache.borrow().get(key.as_str()).cloned())
    {
        return Ok(font_object);
    }

    let data = system_source::load(face, style)?;
    let font = FontExt::from_data(data);
    FONT_OBJECT_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, font.clone());
    });
    Ok(font)
}

fn cache_key(face: FontFamily<'_>, style: FontStyle) -> String {
    match style {
        FontStyle::Normal => face.as_str().to_owned(),
        _ => format!("{}, {}", face.as_str(), style.as_str()),
    }
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
        let pixel_per_em = (size / 1.24) as f32;
        let font = &self.0;
        let font_ref = font.font_ref();
        let face = font.face();
        let metrics = font_ref.metrics(&[]);
        let glyph_metrics = font_ref.glyph_metrics(&[]).scale(pixel_per_em);
        let charmap = font_ref.charmap();

        let mut x_pixels = 0f32;
        let mut prev = None;
        let place_holder = glyph_for_char(&charmap, PLACEHOLDER_CHAR);

        for c in text.chars() {
            if let Some(glyph_id) = glyph_for_char(&charmap, c).or(place_holder) {
                if let Some(pc) = prev {
                    x_pixels += scale_design_units(
                        kerning_units(&face, pc, glyph_id) as f32,
                        pixel_per_em,
                        metrics.units_per_em,
                    );
                }
                x_pixels += glyph_metrics.advance_width(glyph_id);
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
        let font_ref = font.font_ref();
        let face = font.face();
        let metrics = font_ref.metrics(&[]);
        let glyph_metrics = font_ref.glyph_metrics(&[]).scale(em);
        let charmap = font_ref.charmap();

        // Place the swash pen at the baseline. font-kit rasterized into a
        // `size`-square canvas whose top sat at `pos.y - 0.24*em`, then
        // applied a `(0, em)` rasterization translation, putting the
        // effective baseline at `pos.y + 0.76*em`. Swash places glyphs
        // relative to the pen directly, so we shift the pen to that same
        // baseline; otherwise glyphs render ~one em above where callers
        // expect them.
        base_y += (0.76 * em) as i32;

        let mut prev = None;
        let place_holder = glyph_for_char(&charmap, PLACEHOLDER_CHAR);

        let draw_result = SCALE_CONTEXT.with(|scale_context| {
            let mut scale_context = scale_context.borrow_mut();
            let mut scaler = scale_context
                .builder_with_id(font_ref, font.cache_id())
                .size(em)
                .hint(true)
                .build();
            let mut renderer = Render::new(&RENDER_SOURCES);
            renderer.format(Format::Alpha);

            for c in text.chars() {
                if let Some(glyph_id) = glyph_for_char(&charmap, c).or(place_holder) {
                    if let Some(pc) = prev {
                        x += scale_design_units(
                            kerning_units(&face, pc, glyph_id) as f32,
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
                                draw(
                                    base_x + image.placement.left + dx as i32,
                                    base_y - image.placement.top + dy as i32,
                                    alpha,
                                )?
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
        let a = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        let b = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        assert!(
            Arc::ptr_eq(&a.bytes, &b.bytes),
            "cached loads should share the underlying byte buffer"
        );
        assert_eq!(a.id, b.id, "cached loads should share font id");
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
        let size = 32.0_f64;
        let em = (size / 1.24) as f32;
        let pos_y = size as i32;
        // Baseline must match the pen position chosen in `draw`.
        let baseline = pos_y + (0.76 * em) as i32;
        let font = FontDataInternal::new(family, style)?;
        let mut samples = Vec::new();

        let draw_result = font.draw((0, pos_y), size, "Hg", |x, y, alpha| {
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

        // Baseline-anchored bounds. The pen is at output y = `baseline`,
        // ascenders extend up by ~em and descenders down by ~0.3*em.
        assert!(min_x >= 0, "glyphs drifted left: min_x={}", min_x);
        assert!(
            min_y >= baseline - (1.2 * em) as i32,
            "glyphs drifted too high above baseline {}: min_y={}",
            baseline,
            min_y
        );
        assert!(
            max_x <= (3.0 * em) as i32,
            "glyphs drifted right: max_x={}",
            max_x
        );
        assert!(
            max_y <= baseline + (0.6 * em) as i32,
            "glyphs drifted too far below baseline {}: max_y={}",
            baseline,
            max_y
        );

        // 'g' descender must land below the baseline; if placement.top were
        // added rather than subtracted, every glyph would render above
        // baseline and this would fail.
        assert!(
            max_y > baseline,
            "expected 'g' descender below baseline {}: max_y={}",
            baseline,
            max_y
        );

        // Cap height should sit above the baseline by a meaningful amount.
        assert!(
            min_y < baseline,
            "expected glyph tops above baseline {}: min_y={}",
            baseline,
            min_y
        );

        // The touched bbox should span roughly one em vertically; this
        // guards against placement.top being applied with the wrong scale.
        let bbox_height = (max_y - min_y) as f32;
        assert!(
            (0.5 * em..=1.5 * em).contains(&bbox_height),
            "bbox height {} not within [0.5*em, 1.5*em] (em={})",
            bbox_height,
            em
        );

        // 'g' is the second glyph; it must be drawn well to the right of
        // 'H'. Catches a missing advance_width or placement.left bug.
        assert!(
            max_x > (0.6 * em) as i32,
            "expected second glyph drawn after 'H'; max_x={}",
            max_x
        );

        Ok(())
    }
}
