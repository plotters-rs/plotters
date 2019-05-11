use plotters::prelude::*;
use wasm_bindgen::prelude::*;

fn start_plotting(element:&str) -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = CanvasBackend::new(element).unwrap();
    backend.open()?;
    let root: DrawingArea<_, _> = backend.into();
    let font = Into::<FontDesc>::into("Arial").resize(20.0);
    root.fill(&RGBColor(255, 255, 255))?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", &font)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(-1f32..1f32, 0f32..1f32);

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
        &RGBColor(255, 0, 0),
    ))?;

    root.close()?;
    return Ok(());
}

#[wasm_bindgen]
pub fn draw(element:&str) -> bool {
    return start_plotting(element).is_ok();
}
