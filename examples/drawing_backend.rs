use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let mut backend = BitMapBackend::new("examples/outputs/1.png", (800,600));
    // And if we want SVG backend
    // let backend = SVGBackend::new("output.svg", (800, 600));
    backend.open()?;
    backend.draw_rect((100,100), (500, 500), &RGBColor(255,0,0), true)?;
    backend.close()?;
    return Ok(());
}
