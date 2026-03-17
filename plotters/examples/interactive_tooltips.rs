//! Interactive SVG tooltips examples.
//!
//! This example demonstrates the `draw_series_with_tooltips` API together with
//! `SVGBackend::with_tooltips()`. The generated SVG file contains embedded CSS and JavaScript that
//! show a tooltip on hover for every data point.
//!
//! Open `plotters-doc-data/interactive_tooltips.svg` in a browser to see it in action.

use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "plotters-doc-data/interactive_tooltips.svg";

    // Make sure the output directory exists
    std::fs::create_dir_all("plotters-doc-data")?;

    let root = SVGBackend::new(path, (720, 460))
        .with_tooltips()
        .into_drawing_area();

    root.fill(&WHITE);

    let mut chart = ChartBuilder::on(&root)
        .caption("Interactive Tooltips Demo", ("sans-serif", 28).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..6.5, -1.2..1.2)?;

    chart.configure_mesh().x_desc("X").y_desc("Y").draw()?;

    // --- Series 1: sin(x) ---
    let sin_data: Vec<_> = (0..=60)
        .map(|i| {
            let x = i as f64 / 10.;
            (x, x.sin())
        })
        .collect();

    chart
        .draw_series_with_tooltips(
            LineSeries::new(sin_data.iter().copied(), &RED),
            &RED,
            "sin(x)",
        )?
        .label("sin(x)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    // --- Series 2: cos(x) ---
    let cos_data: Vec<_> = (0..=60)
        .map(|i| {
            let x = i as f64 / 10.;
            (x, x.cos())
        })
        .collect();

    chart
        .draw_series_with_tooltips(
            LineSeries::new(cos_data.iter().copied(), &BLUE).point_size(3),
            &BLUE,
            "cos(x)",
        )?
        .label("cos(x)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;

    println!("Chart saved to {path}");
    println!("Open it in a web browser to see interactive tooltips on hover");

    Ok(())
}
