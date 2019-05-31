use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("examples/outputs/3.png", (300, 200)).into_drawing_area();
    root.fill(&White)?;
    // Draw an circle on the drawing area
    root.draw(&Circle::new(
        (100, 100),
        50,
        Into::<ShapeStyle>::into(&Green).filled(),
    ))?;
    Ok(())
}
