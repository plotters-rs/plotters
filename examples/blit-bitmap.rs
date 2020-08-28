use plotters::prelude::*;

use image::{imageops::FilterType, ImageFormat};

use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        BitMapBackend::new("plotters-doc-data/blit-bitmap.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Bitmap Example", ("sans-serif", 30))
        .margin(5)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0.0..1.0, 0.0..1.0)?;

    chart.configure_mesh().disable_mesh().draw()?;

    let (w, h) = chart.plotting_area().dim_in_pixel();
    let image = image::load(
        BufReader::new(File::open("plotters-doc-data/cat.png")?),
        ImageFormat::Png,
    )?
    .resize_exact(w - w / 10, h - h / 10, FilterType::Nearest);

    let elem: BitMapElement<_> = ((0.05, 0.95), image).into();

    chart.draw_series(std::iter::once(elem))?;
    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
