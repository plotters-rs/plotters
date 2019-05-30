use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = BitMapBackend::new("examples/outputs/histogram.png", (640, 480));
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 50.0).into();
    root.fill(&RGBColor(255, 255, 255))?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(25)
        .y_label_area_size(40)
        .caption("Histogram Test", &font)
        .build_ranged(0u32..10u32, 0u32..10u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .line_style_1(&RGBColor(255, 255, 255).mix(0.3))
        .x_label_offset(30)
        .draw()?;

    let data = [
        0, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
    ];

    chart.draw_series(Histogram::<RangedCoordu32, _>::new(
        data.iter().map(|x: &u32| (*x, 1u32)),
        5,
        Into::<ShapeStyle>::into(&RGBColor(255, 0, 0).mix(0.5)).filled(),
    ))?;

    Ok(())
}
