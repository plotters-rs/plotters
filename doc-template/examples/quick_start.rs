use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("examples/outputs/0.png", (640, 480)).into_drawing_area();
    root.fill(&White)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", &("Arial", 50).into_font())
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(-1f32..1f32, -0.1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
        &RGBColor(255, 0, 0),
    ))?;
    Ok(())
}
