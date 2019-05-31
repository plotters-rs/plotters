use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let mut backend = BitMapBackend::new("examples/outputs/1.png", (300, 200));
    // And if we want SVG backend
    // let backend = SVGBackend::new("output.svg", (800, 600));
    backend.draw_rect((50, 50), (200, 150), &Red, true)?;
    Ok(())
}
