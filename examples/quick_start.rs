use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new("examples/outputs/0.png", (640, 480));
    backend.open()?;
    let root: DrawingArea<_, _> = backend.into();
    let font = Into::<FontDesc>::into("DejaVu Serif").resize(20.0);
    root.fill(&RGBColor(255, 255, 255))?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", &font)
        .build_ranged::<RangedCoordf32, RangedCoordf32, _, _>(-1f32..1f32, 0f32..1f32);

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
        &RGBColor(255, 0, 0),
    ))?;

    root.close()?;
    return Ok(());
}
