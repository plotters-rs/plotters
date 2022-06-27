use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("../target/plotters-doc-data")?;
    let root = BitMapBackend::new("../target/plotters-doc-data/3.png", (300, 200)).into_drawing_area();
    root.fill(&WHITE)?;
    // Draw an circle on the drawing area
    root.draw(&Circle::new(
        (100, 100),
        50,
        Into::<ShapeStyle>::into(&GREEN).filled(),
    ))?;
    root.present()?;
    Ok(())
}
