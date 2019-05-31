use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_drawing_area =
        BitMapBackend::new("examples/outputs/2.png", (300, 200)).into_drawing_area();
    // And we can split the drawing area into 3x3 grid
    let child_drawing_areas = root_drawing_area.split_evenly((3, 3));
    // Then we fill the drawing area with different color
    for (area, color) in child_drawing_areas.into_iter().zip(0..) {
        area.fill(&Palette99::pick(color))?;
    }
    Ok(())
}
