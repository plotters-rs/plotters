use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("examples/outputs/histogram.png", (640, 480)).into_drawing_area();

    root.fill(&White)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Histogram Test", ("Arial", 50.0).into_font())
        .build_ranged(0u32..10u32, 0u32..10u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .line_style_1(&White.mix(0.3))
        .x_label_offset(30)
        .y_desc("Count")
        .x_desc("Bucket")
        .axis_desc_style(("Arial", 15).into_font())
        .draw()?;

    let data = [
        0u32, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
    ];

    chart.draw_series(Histogram::<RangedCoordu32, _>::new(
        data.iter().map(|x: &u32| (*x, 1u32)),
        5,
        ShapeStyle::from(&Red.mix(0.5)).filled(),
    ))?;

    Ok(())
}
