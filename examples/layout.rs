use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "plotters-doc-data/layout2.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 600;
    const H: u32 = 400;
    let root = BitMapBackend::new(OUT_FILE_NAME, (W, H)).into_drawing_area();
    root.fill(&full_palette::WHITE)?;

    let mut chart = ChartLayout::new(&root);
    chart.set_title_text("Chart Title").unwrap();
    chart.draw().unwrap();

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
