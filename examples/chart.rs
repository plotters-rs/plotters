use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_area =
        BitMapBackend::new("examples/outputs/sample.png", (1024, 768)).into_drawing_area();

    root_area.fill(&White)?;

    let root_area = root_area
        .titled("Image Title", &("Arial", 60).into_font())?
        .margin(0, 0, 0, 20);

    let (upper, lower) = root_area.split_vertically(512);

    let mut cc = ChartBuilder::on(&upper)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .caption("Sine and Cosine", &("Arial", 40).into_font())
        .build_ranged(-3.4f32..3.4f32, -1.2f32..1.2f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_x_mesh()
        .disable_y_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    cc.draw_series(LineSeries::new(
        (0..12).map(|x| ((x - 6) as f32 / 2.0, ((x - 6) as f32 / 2.0).sin())),
        &Red,
    ))?
    .label("Sine")
    .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &Red));

    cc.draw_series(LineSeries::new(
        (0..6800).map(|x| {
            (
                (x - 3400) as f32 / 1000.0,
                ((x - 3400) as f32 / 1000.0).cos(),
            )
        }),
        &Blue,
    ))?
    .label("Cosine")
    .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &Blue));

    cc.configure_series_labels().border_style(&Black).draw()?;

    /*
    // It's possible to use a existing pointing element
     cc.draw_series(PointSeries::<_, _, Circle<_>>::new(
        (0..6).map(|x| ((x - 3) as f32 / 1.0, ((x - 3) as f32 / 1.0).sin())),
        5,
        Into::<ShapeStyle>::into(&RGBColor(255,0,0)).filled(),
    ))?;*/

    // Otherwise you can use a function to construct your pointing element yourself
    let point_font = ("Arial", 15).into_font();
    cc.draw_series(PointSeries::of_element(
        (0..6).map(|x| ((x - 3) as f32 / 1.0, ((x - 3) as f32 / 1.0).sin())),
        5,
        ShapeStyle::from(&Red).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + Text::new(format!("{:?}", coord), (0, 15), &point_font)
        },
    ))?;

    let drawing_areas = lower.split_evenly((1, 2));

    for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .caption(format!("y = x^{}", 1 + 2 * idx), &("Arial", 40).into_font())
            .build_ranged(-1f32..1f32, -1f32..1f32)?;
        cc.configure_mesh().x_labels(5).y_labels(3).draw()?;

        cc.draw_series(LineSeries::new(
            (-100..100).map(|x| {
                (
                    x as f32 / 100.0,
                    (x as f32 / 100.0).powf(idx as f32 * 2.0 + 1.0),
                )
            }),
            &Blue,
        ))?;
    }

    Ok(())
}
