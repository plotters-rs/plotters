//! Demonstrates `DrawingArea::with_fonts` by attaching multiple fonts as raw
//! byte buffers, without touching `register_font`, global state, or the host's
//! installed fonts. The chart renders exactly the bytes that were handed in.
//!
//! The .ttf files for Roboto Regular, Roboto Bold, and Kablammo Regular are
//! checked into `examples/fonts/` (both are SIL OFL 1.1; see the .txt files
//! alongside) so this example also works in CI environments without network
//! access.
//!
//! To pull the same fonts off the network instead — e.g. for a real app that
//! caches Google Fonts at startup — the CSS endpoint below will return TTF
//! URLs when called with a non-browser user-agent like `Wget/1.20`. Browser
//! UAs get WOFF2 / WOFF / EOT, which harfrust + skrifa do not parse.
//!
//! ```text
//! GET https://fonts.googleapis.com/css2?family=Kablammo&family=Roboto:ital,wght@0,100..900;1,100..900&display=swap
//! User-Agent: Wget/1.20
//! ```
//!
//! Walk the @font-face blocks, pick the URL whose font-family / font-style /
//! font-weight match what you need, GET that .ttf, and feed the bytes into
//! `with_fonts` exactly the way this example does with the bundled buffers.
//!
//! Run with:
//!
//! ```text
//! cargo run --example dynamic_font --release
//! ```

use plotters::prelude::*;
use std::error::Error;
use std::sync::Arc;

const ROBOTO_REGULAR: &[u8] = include_bytes!("fonts/Roboto-Regular.ttf");
const ROBOTO_BOLD: &[u8] = include_bytes!("fonts/Roboto-Bold.ttf");
const KABLAMMO_REGULAR: &[u8] = include_bytes!("fonts/Kablammo-Regular.ttf");

const OUT_FILE_NAME: &str = "plotters-doc-data/dynamic_font.png";

fn main() -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (640, 480))
        .into_drawing_area()
        .with_fonts([
            ("Roboto", FontStyle::Normal, Arc::<[u8]>::from(ROBOTO_REGULAR)),
            ("Roboto", FontStyle::Bold, Arc::<[u8]>::from(ROBOTO_BOLD)),
            ("Kablammo", FontStyle::Normal, Arc::<[u8]>::from(KABLAMMO_REGULAR)),
        ]);
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Hello from Plotters!", ("Kablammo", 36))
        .margin(20)
        .x_label_area_size(45)
        .y_label_area_size(55)
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart
        .configure_mesh()
        .x_desc("Time elapsed (seconds)")
        .y_desc("Distance fallen (meters)")
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
