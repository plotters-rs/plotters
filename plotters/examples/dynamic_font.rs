//! Demonstrates `DrawingArea::with_fonts` by attaching multiple fonts that
//! were downloaded at runtime instead of installed on the host or registered
//! through the legacy `register_font` API.
//!
//! The example pulls Roboto and Kablammo from Google Fonts in a single CSS
//! request, picks the matching .ttf URLs out of the payload, fetches each
//! file, and hands the byte buffers to the new font pipeline keyed by the
//! family name the chart code will reference. Nothing touches
//! `register_font`, no global state is mutated, and system fonts are never
//! consulted -- the chart renders exactly the bytes that were downloaded.
//!
//! Run with:
//!
//! ```text
//! cargo run --example dynamic_font --release
//! ```

use plotters::prelude::*;
use std::error::Error;
use std::io::Read;
use std::sync::Arc;

const FONTS_CSS_URL: &str = "https://fonts.googleapis.com/css2\
    ?family=Kablammo\
    &family=Roboto:ital,wght@0,100..900;1,100..900\
    &display=swap";

// Browser user-agents get WOFF2 / WOFF / EOT from Google Fonts depending on
// vintage; harfrust + skrifa read raw OpenType. A Wget user-agent reliably
// receives TTF URLs on the css2 endpoint.
const TTF_UA: &str = "Wget/1.20";

const OUT_FILE_NAME: &str = "plotters-doc-data/dynamic_font.png";

fn main() -> Result<(), Box<dyn Error>> {
    let css = http_get_text(FONTS_CSS_URL, TTF_UA)?;
    let roboto_regular = fetch_ttf(&css, "Roboto", "normal", 400)?;
    let roboto_bold = fetch_ttf(&css, "Roboto", "normal", 700)?;
    let kablammo = fetch_ttf(&css, "Kablammo", "normal", 400)?;

    let root = BitMapBackend::new(OUT_FILE_NAME, (640, 480))
        .into_drawing_area()
        .with_fonts([
            ("Roboto", FontStyle::Normal, roboto_regular),
            ("Roboto", FontStyle::Bold, roboto_bold),
            ("Kablammo", FontStyle::Normal, kablammo),
        ]);
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("y = x²", ("Kablammo", 32))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart
        .configure_mesh()
        .x_desc("x")
        .y_desc("y")
        .label_style(("Roboto", 14))
        .axis_desc_style(("Roboto", 16, FontStyle::Bold))
        .draw()?;

    chart.draw_series(LineSeries::new(
        (0..=100).map(|i| {
            let x = i as f32 / 10.0;
            (x, x * x)
        }),
        &BLUE,
    ))?;

    root.present()?;
    println!("rendered {OUT_FILE_NAME}");
    Ok(())
}

fn http_get_text(url: &str, user_agent: &str) -> Result<String, Box<dyn Error>> {
    Ok(ureq::get(url)
        .set("User-Agent", user_agent)
        .call()?
        .into_string()?)
}

fn http_get_bytes(url: &str) -> Result<Arc<[u8]>, Box<dyn Error>> {
    let mut buf = Vec::new();
    ureq::get(url).call()?.into_reader().read_to_end(&mut buf)?;
    Ok(Arc::from(buf.into_boxed_slice()))
}

/// Walks the @font-face blocks in a Google Fonts CSS payload and downloads
/// the .ttf URL whose font-family, font-style, and font-weight match the
/// request. Combined CSS responses can return many candidate weights per
/// family (Kablammo's variable axis, Roboto's full 100..900 range), so the
/// first matching block wins.
fn fetch_ttf(
    css: &str,
    family: &str,
    style: &str,
    weight: u32,
) -> Result<Arc<[u8]>, Box<dyn Error>> {
    for block in css.split("@font-face").skip(1) {
        let block = &block[..block.find('}').unwrap_or(block.len())];

        if read_field(block, "font-family") != Some(family) {
            continue;
        }
        if read_field(block, "font-style") != Some(style) {
            continue;
        }
        let block_weight = read_field(block, "font-weight")
            .and_then(|raw| raw.split_whitespace().next())
            .and_then(|raw| raw.parse::<u32>().ok());
        if block_weight != Some(weight) {
            continue;
        }

        if let Some(url) = first_ttf_url(block) {
            return http_get_bytes(url);
        }
    }
    Err(format!("no .ttf for {family} {style} {weight} in CSS — try a different User-Agent").into())
}

fn read_field<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}:");
    let start = block.find(&key)? + key.len();
    let after = &block[start..];
    let end = after.find(';')?;
    Some(after[..end].trim().trim_matches('\''))
}

fn first_ttf_url(block: &str) -> Option<&str> {
    let mut rest = block;
    while let Some(idx) = rest.find("url(") {
        let after = &rest[idx + 4..];
        let end = after.find(')')?;
        let url = after[..end].trim_matches(|c: char| matches!(c, '\'' | '"' | ' '));
        if url.ends_with(".ttf") {
            return Some(url);
        }
        rest = &after[end + 1..];
    }
    None
}
