use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "plotters-doc-data/layout2.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const W: u32 = 600;
    const H: u32 = 400;
    let root = BitMapBackend::new(OUT_FILE_NAME, (W, H)).into_drawing_area();
    root.fill(&full_palette::WHITE)?;

    let mut chart = ChartLayout::new(&root);
    chart
        .set_chart_title_text("Chart Title")?
        .set_top_label_text("A label at the top")?
        .set_chart_title_style(("serif", 60.).into_font().with_color(&RED))?
        .set_left_label_text("Left label")?
        .set_right_label_text("Right label")?
        .set_bottom_label_text("Bottom label")?
        .set_bottom_label_margin(10.)?
        .set_top_label_margin(10.)?
    .draw()?;

    let extent = chart.get_chart_area_extent()?;
    root.draw(&Rectangle::new(extent.into_array_of_tuples(), &BLUE))?;
    let extent = chart.get_chart_title_extent()?;
    root.draw(&Rectangle::new(extent.into_array_of_tuples(), &BLUE))?;
    //dbg!();

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
