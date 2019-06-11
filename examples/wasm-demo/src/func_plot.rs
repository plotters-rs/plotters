use plotters::prelude::*;
use wasm_bindgen::prelude::*;

fn start_plotting(
    element: &str,
    pow: i32,
) -> Result<Box<dyn Fn((i32, i32)) -> Option<(f32, f32)>>, Box<dyn std::error::Error>> {
    let backend = CanvasBackend::new(element).unwrap();
    let root = backend.into_drawing_area();
    let font: FontDesc = ("Arial", 20.0).into();

    root.fill(&White)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("y=x^{}", pow), font)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(-1f32..1f32, -1.2f32..1.2f32)?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw()?;

    chart.draw_series(LineSeries::new(
        (-50..=50)
            .map(|x| x as f32 / 50.0)
            .map(|x| (x, x.powf(pow as f32))),
        &Red,
    ))?;

    root.present()?;
    return Ok(Box::new(chart.into_coord_trans()));
}

#[wasm_bindgen]
pub fn draw_func(element: &str, p: i32) -> JsValue {
    crate::make_coord_mapping_closure(start_plotting(element, p).ok())
}
