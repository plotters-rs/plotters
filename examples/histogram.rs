use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new("examples/outputs/histogram.png", (640, 480));
    backend.open()?;
    let root: DrawingArea<_, _> = backend.into();
    let font = Into::<FontDesc>::into("DejaVu Serif").resize(20.0);
    root.fill(&RGBColor(255, 255, 255))?;
    
    let mut chart = ChartBuilder::on(&root)
        .set_x_label_size(40)
        .set_y_label_size(40)
        .caption("Histogram Test", &font)
        .build_ranged::<RangedCoordu32, RangedCoordu32, _, _>(0..10, 0..10);

    chart.configure_mesh().draw()?;

    let data = [0,1,1,1,4,2,5,7,8,6,4,2,1,8,3,3,3,4,4,3,3,3,];

    chart.draw_series(Histogram::<RangedCoordu32,_>::new(data.iter().map(|x:&u32| (*x,1u32)), 5, Into::<ShapeStyle>::into(&RGBColor(255,0,0).mix(0.5)).filled()))?;

    root.close()?;
    return Ok(());
}
