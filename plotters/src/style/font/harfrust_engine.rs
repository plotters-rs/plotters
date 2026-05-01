use super::engine::{CoverageMask, FontEngine, FontError, ParsedFont, PositionedGlyph, ShapedRun};
use harfrust::{Direction, FontRef as HarfrustFontRef, ShaperData, UnicodeBuffer};
use skrifa::outline::{
    DrawSettings, Engine, HintingInstance, HintingOptions, OutlineGlyph, OutlinePen,
};
use skrifa::prelude::{LocationRef, Size};
use skrifa::{FontRef as SkrifaFontRef, MetadataProvider};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zeno::{Command, Mask, PathBuilder};

#[derive(Default)]
pub struct HarfrustEngine;

impl FontEngine for HarfrustEngine {
    fn parse(&self, data: Arc<[u8]>, index: u32) -> Result<Arc<dyn ParsedFont>, FontError> {
        HarfrustFontRef::from_index(data.as_ref(), index)
            .map_err(|err| FontError::InvalidFontData(err.to_string()))?;
        SkrifaFontRef::from_index(data.as_ref(), index)
            .map_err(|err| FontError::InvalidFontData(err.to_string()))?;

        Ok(Arc::new(HarfrustFont {
            data,
            index,
            hinters: Mutex::new(HashMap::new()),
        }))
    }
}

struct HarfrustFont {
    data: Arc<[u8]>,
    index: u32,
    // Building a HintingInstance traces the font's TrueType bytecode
    // interpreter; doing it on every rasterize call dwarfs the actual
    // outline drawing. Cache by quantized pixel size.
    hinters: Mutex<HashMap<u32, Option<Arc<HintingInstance>>>>,
}

impl HarfrustFont {
    fn harfrust_font(&self) -> Result<HarfrustFontRef<'_>, FontError> {
        HarfrustFontRef::from_index(self.data.as_ref(), self.index)
            .map_err(|_| FontError::InvalidFontIndex(self.index))
    }

    fn skrifa_font(&self) -> Result<SkrifaFontRef<'_>, FontError> {
        SkrifaFontRef::from_index(self.data.as_ref(), self.index)
            .map_err(|_| FontError::InvalidFontIndex(self.index))
    }

    fn hinter_for(&self, size_px: f32) -> Result<Option<Arc<HintingInstance>>, FontError> {
        let key = size_px.to_bits();
let hinter = self.hinters.lock()
    .map_err(|_| FontError::LockError)?
    .get(&key)
    .cloned();

if let Some(h) = hinter {
    return Ok(h);
}

        let font = self.skrifa_font()?;
        let outlines = font.outline_glyphs();
        // Use the autohinter unconditionally rather than the default
        // AutoFallback. Many TrueType hint programs (Roboto's included)
        // intentionally relax overshoot snapping above ~24px, leaving curved
        // glyphs like c/o/e a fractional pixel below the baseline of flat
        // ones like l/m/i. That is correct typographic behaviour but reads
        // as a baseline bug in chart labels. The autohinter snaps overshoots
        // at every size so labels keep a single shared baseline.
        let hinter = HintingInstance::new(
            &outlines,
            Size::new(size_px),
            LocationRef::default(),
            HintingOptions {
                engine: Engine::Auto(None),
                ..HintingOptions::default()
            },
        )
        // Treat hinting as an optimisation: if it fails for some reason
        // (corrupt instructions, exotic font features) the unhinted outlines
        // are still valid, so cache the miss and fall back at draw time.
        .ok()
        .map(Arc::new);
        self.hinters
            .lock()
            .map_err(|_| FontError::LockError)?
            .insert(key, hinter.clone());
        Ok(hinter)
    }
}

impl ParsedFont for HarfrustFont {
    fn shape(&self, text: &str, size_px: f32) -> Result<ShapedRun, FontError> {
        if text.is_empty() {
            return Ok(ShapedRun {
                glyphs: Vec::new(),
                bounds: ((0, 0), (0, 0)),
            });
        }

        let font = self.harfrust_font()?;
        let shaper_data = ShaperData::new(&font);
        let shaper = shaper_data.shaper(&font).point_size(Some(size_px)).build();
        let scale = size_px / (shaper.units_per_em().max(1) as f32);

        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);

        let shaped = shaper.shape(buffer, &[]);
        let infos = shaped.glyph_infos();
        let positions = shaped.glyph_positions();
        let mut glyphs = Vec::with_capacity(infos.len());
        let mut cursor_x = 0.0f32;
        let mut cursor_y = 0.0f32;

        for (info, position) in infos.iter().zip(positions) {
            glyphs.push(PositionedGlyph {
                id: info.glyph_id,
                x: (cursor_x + position.x_offset as f32) * scale,
                y: -(cursor_y + position.y_offset as f32) * scale,
            });
            cursor_x += position.x_advance as f32;
            cursor_y += position.y_advance as f32;
        }

let font = self.skrifa_font()?;
let metrics = font.metrics(Size::new(size_px), LocationRef::default());
let min_y = (-metrics.ascent).floor() as i32;
let descent_y = (-metrics.descent).ceil() as i32;
let max_y = if descent_y > min_y { descent_y } else { size_px.ceil() as i32 };
let width = (cursor_x * scale).ceil().max(0.0) as i32;

        Ok(ShapedRun {
            glyphs,
            bounds: ((0, min_y), (width, max_y)),
        })
    }

    fn rasterize(
        &self,
        glyph_id: u32,
        size_px: f32,
        subpixel: (f32, f32),
    ) -> Result<CoverageMask, FontError> {
        let font = self.skrifa_font()?;
        let outlines = font.outline_glyphs();
        let Some(glyph) = outlines.get(skrifa::GlyphId::new(glyph_id)) else {
            return Ok(empty_mask());
        };

        let mut path = Vec::new();
        if let Some(hinter) = self.hinter_for(size_px)? {
            if glyph
                .draw(
                    DrawSettings::hinted(&hinter, false),
                    &mut ZenoPen {
                        path: &mut path,
                        sx: subpixel.0,
                        sy: subpixel.1,
                    },
                )
                .is_err()
            {
                path.clear();
                draw_unhinted_glyph(&glyph, size_px, subpixel, &mut path)?;
            }
        } else {
            draw_unhinted_glyph(&glyph, size_px, subpixel, &mut path)?;
        }

        if path.is_empty() {
            return Ok(empty_mask());
        }

        let (data, placement) = Mask::new(&path).render();
        Ok(CoverageMask {
            left: placement.left,
            top: placement.top,
            width: placement.width,
            height: placement.height,
            data,
        })
    }
}

fn draw_unhinted_glyph(
    glyph: &OutlineGlyph<'_>,
    size_px: f32,
    subpixel: (f32, f32),
    path: &mut Vec<Command>,
) -> Result<(), FontError> {
    glyph
        .draw(
            DrawSettings::unhinted(Size::new(size_px), LocationRef::default()),
            &mut ZenoPen {
                path,
                sx: subpixel.0,
                sy: subpixel.1,
            },
        )
        .map(|_| ())
        .map_err(|err| FontError::RasterizeError(err.to_string()))
}

fn empty_mask() -> CoverageMask {
    CoverageMask {
        left: 0,
        top: 0,
        width: 0,
        height: 0,
        data: Vec::new(),
    }
}

struct ZenoPen<'a> {
    path: &'a mut Vec<Command>,
    // Subpixel offset folded into the path coordinates so the rendered mask
    // already accounts for the glyph's fractional pixel position. Without
    // this, the caller has to round to integer pixel positions and the
    // sub-pixel kerning information from harfrust is lost.
    sx: f32,
    sy: f32,
}

impl OutlinePen for ZenoPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to((x + self.sx, -y + self.sy));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to((x + self.sx, -y + self.sy));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.path
            .quad_to((cx0 + self.sx, -cy0 + self.sy), (x + self.sx, -y + self.sy));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.path.curve_to(
            (cx0 + self.sx, -cy0 + self.sy),
            (cx1 + self.sx, -cy1 + self.sy),
            (x + self.sx, -y + self.sy),
        );
    }

    fn close(&mut self) {
        self.path.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/SourceSansPro-Regular-Tiny.ttf");

    #[test]
    fn shapes_and_rasterizes_fixture_font() {
        let engine = HarfrustEngine;
        let font = engine.parse(Arc::<[u8]>::from(FONT_BYTES), 0).unwrap();

        let run = font.shape("Hello", 24.0).unwrap();
        assert!(!run.glyphs.is_empty());
        let ((min_x, min_y), (max_x, max_y)) = run.bounds;
        assert!(max_x > min_x);
        assert!(max_y > min_y);

        let mask = run
            .glyphs
            .iter()
            .map(|glyph| font.rasterize(glyph.id, 24.0, (0.0, 0.0)).unwrap())
            .find(|mask| mask.width > 0 && mask.height > 0)
            .expect("at least one glyph has an outline");

        assert_eq!(mask.data.len(), (mask.width * mask.height) as usize);
        assert!(mask.data.iter().any(|alpha| *alpha > 0));
    }

    #[test]
    fn subpixel_offset_changes_mask_data() {
        let engine = HarfrustEngine;
        let font = engine.parse(Arc::<[u8]>::from(FONT_BYTES), 0).unwrap();
        let glyph_id = font.shape("H", 18.0).unwrap().glyphs[0].id;

        let aligned = font.rasterize(glyph_id, 18.0, (0.0, 0.0)).unwrap();
        let shifted = font.rasterize(glyph_id, 18.0, (0.5, 0.0)).unwrap();

        // A half-pixel horizontal shift either changes the placement or the
        // coverage values; otherwise subpixel positioning is being dropped.
        let same_placement = aligned.left == shifted.left
            && aligned.top == shifted.top
            && aligned.width == shifted.width
            && aligned.height == shifted.height;
        assert!(!same_placement || aligned.data != shifted.data);
    }
}
