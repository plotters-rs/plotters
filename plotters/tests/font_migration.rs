#![cfg(all(
    feature = "bitmap_backend",
    not(all(target_arch = "wasm32", not(target_os = "wasi")))
))]

use plotters::coord::Shift;
use plotters::prelude::*;
use std::sync::Arc;
use std::thread;

static FONT_BYTES: &[u8] = include_bytes!("fixtures/SourceSansPro-Regular-Tiny.ttf");

const CANVAS: (u32, u32) = (180, 90);

fn buffer() -> Vec<u8> {
    vec![255; (CANVAS.0 * CANVAS.1 * 3) as usize]
}

fn font_bytes() -> Arc<[u8]> {
    Arc::<[u8]>::from(FONT_BYTES)
}

fn root<'a>(buffer: &'a mut [u8]) -> DrawingArea<BitMapBackend<'a>, Shift> {
    BitMapBackend::with_buffer(buffer, CANVAS).into_drawing_area()
}

fn font_table(name: &'static str) -> Vec<(&'static str, FontStyle, Arc<[u8]>)> {
    vec![(name, FontStyle::Normal, font_bytes())]
}

fn style(name: &'static str) -> TextStyle<'static> {
    (name, 28).into_font().color(&BLACK)
}

fn assert_has_ink(buffer: &[u8]) {
    let inked_pixels = buffer
        .chunks_exact(3)
        .filter(|pixel| pixel.iter().any(|&channel| channel != 255))
        .count();
    assert!(inked_pixels > 0, "expected rendered text to change pixels");
}

fn assert_text_missing(area: &DrawingArea<BitMapBackend<'_>, Shift>, family: &'static str) {
    let err = area
        .estimate_text_size("Hello", &style(family))
        .expect_err("text should not resolve outside the active context");
    let message = err.to_string();
    assert!(
        message.contains("font is not in context") || message.contains("system fonts are disabled"),
        "unexpected missing-font error: {}",
        message
    );
}

#[test]
fn with_fonts_draws_text_to_bitmap() {
    const FAMILY: &str = "PlottersFixtureWithFonts";

    let mut pixels = buffer();
    {
        let area = root(&mut pixels).with_fonts(font_table(FAMILY));
        area.draw_text("Hello", &style(FAMILY), (8, 8)).unwrap();
    }

    assert_has_ink(&pixels);
}

#[test]
fn explicit_context_isolated_from_default_area() {
    const FAMILY: &str = "PlottersFixtureExplicitOnly";

    let mut explicit_pixels = buffer();
    {
        let area = root(&mut explicit_pixels).with_fonts(font_table(FAMILY));
        area.draw_text("Hello", &style(FAMILY), (8, 8)).unwrap();
    }
    assert_has_ink(&explicit_pixels);

    let mut default_pixels = buffer();
    {
        let area = root(&mut default_pixels);
        assert_text_missing(&area, FAMILY);
    }
}

#[test]
fn sub_areas_inherit_parent_context() {
    const FAMILY: &str = "PlottersFixtureInherited";

    let mut pixels = buffer();
    {
        let area = root(&mut pixels).with_fonts(font_table(FAMILY));
        let children = area.split_evenly((1, 2));
        children[1]
            .draw_text("Hello", &style(FAMILY), (8, 8))
            .unwrap();
    }

    assert_has_ink(&pixels);
}

#[test]
fn sub_area_context_override_stays_local() {
    const PARENT: &str = "PlottersFixtureParent";
    const CHILD: &str = "PlottersFixtureChild";

    let mut pixels = buffer();
    {
        let area = root(&mut pixels).with_fonts(font_table(PARENT));
        let child = area.split_evenly((1, 2))[0]
            .clone()
            .with_fonts(font_table(CHILD));

        assert_text_missing(&child, PARENT);
        child.draw_text("Child", &style(CHILD), (8, 8)).unwrap();
        area.draw_text("Parent", &style(PARENT), (8, 48)).unwrap();
    }

    assert_has_ink(&pixels);
}

#[test]
fn concurrent_drawing_areas_keep_font_contexts_separate() {
    const A: &str = "PlottersFixtureThreadA";
    const B: &str = "PlottersFixtureThreadB";

    let handles = IntoIterator::into_iter([(A, B), (B, A)]).map(|(own, other)| {
        thread::spawn(move || {
            let mut pixels = buffer();
            {
                let area = root(&mut pixels).with_fonts(font_table(own));
                assert_text_missing(&area, other);
                area.draw_text("Hello", &style(own), (8, 8)).unwrap();
            }
            assert_has_ink(&pixels);
        })
    });

    for handle in handles {
        handle.join().unwrap();
    }
}

#[cfg(feature = "ab_glyph")]
#[test]
fn legacy_register_after_area_construction_reaches_default_context() {
    const FAMILY: &str = "PlottersFixtureLegacyLateRegister";

    let mut pixels = buffer();
    {
        let area = root(&mut pixels);
        plotters::style::register_font(FAMILY, FontStyle::Normal, FONT_BYTES).unwrap();
        area.draw_text("Hello", &style(FAMILY), (8, 8)).unwrap();
    }

    assert_has_ink(&pixels);
}

#[cfg(feature = "ab_glyph")]
#[test]
fn with_fonts_does_not_see_legacy_registry() {
    const REGISTERED: &str = "PlottersFixtureLegacyHidden";
    const LOCAL: &str = "PlottersFixtureLocalOnly";

    plotters::style::register_font(REGISTERED, FontStyle::Normal, FONT_BYTES).unwrap();

    let mut pixels = buffer();
    {
        let area = root(&mut pixels).with_fonts(font_table(LOCAL));
        assert_text_missing(&area, REGISTERED);
        area.draw_text("Hello", &style(LOCAL), (8, 8)).unwrap();
    }

    assert_has_ink(&pixels);
}
