use plotters::coord::Shift;
use plotters::prelude::*;

fn draw_chart<B: DrawingBackend>(root: &DrawingArea<B, Shift>) -> DrawResult<(), B> {
    let mut chart = ChartBuilder::on(root)
        .caption(
            "Relative Size Example",
            ("sans-serif", (5).percent_height()),
        )
        .x_label_area_size((10).percent_height())
        .y_label_area_size((10).percent_width())
        .margin(5)
        .build_cartesian_2d(-5.0..5.0, -1.0..1.0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", (3).percent_height()))
        .draw()?;

    chart.draw_series(LineSeries::new(
        (0..1000)
            .map(|x| x as f64 / 100.0 - 5.0)
            .map(|x| (x, x.sin())),
        &RED,
    ))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        BitMapBackend::new("plotters-doc-data/relative_size.png", (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let (left, right) = root.split_horizontally((70).percent_width());

    draw_chart(&left)?;

    let (upper, lower) = right.split_vertically(300);

    draw_chart(&upper)?;
    draw_chart(&lower)?;

    draw_chart(&root.shrink((200, 200), (150, 100)))?;

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
