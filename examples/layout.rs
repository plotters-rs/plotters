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
        .set_chart_title_style(("serif", 60.).into_font().with_color(&RED))?
        .set_left_label_text("Ratio of Sides")?
        .set_right_label_text("Right label")?
        .set_bottom_label_text("Radians")?
        .set_bottom_label_margin(10.)?
        .set_left_label_margin(10.)?
        .set_right_label_margin(10.)?
        .draw()?;

    // If we extract a drawing area corresponding to a chart area, we can
    // use the usual chart API to draw.
    let da_chart = chart.get_chart_drawing_area()?;
    let x_axis = (-3.4f32..3.4).step(0.1);
    let mut cc = ChartBuilder::on(&da_chart)
        .margin(5)
        .set_all_label_area_size(15)
        .build_cartesian_2d(-3.4f32..3.4, -1.2f32..1.2f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, x.sin())), &RED))?
        .label("Sine")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    cc.draw_series(LineSeries::new(
        x_axis.values().map(|x| (x, x.cos())),
        &BLUE,
    ))?
    .label("Cosine")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    cc.configure_series_labels().border_style(&BLACK).draw()?;

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
