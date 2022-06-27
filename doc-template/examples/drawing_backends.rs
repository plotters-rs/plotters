use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("../target/plotters-doc-data")?;
    // Create a 800*600 bitmap and start drawing
    let mut backend = BitMapBackend::new("../target/plotters-doc-data/1.png", (300, 200));
    // And if we want SVG backend
    // let backend = SVGBackend::new("output.svg", (800, 600));
    backend.draw_rect((50, 50), (200, 150), &RED, true)?;
    backend.present()?;
    Ok(())
}
