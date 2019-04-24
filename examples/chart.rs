use plotters::prelude::*;

fn main() {
    let mut img = BitMapBackend::new("/tmp/plotter.png", (1024, 768));

    img.open().unwrap();

    let root_area: DrawingArea<_, _> = img.into();

    root_area.fill(&RGBColor(255, 255, 255)).unwrap();

    let font:FontDesc = "ArialMT".into();
    let font_large = &font.resize(60.0);
    let font_small = &font.resize(40.0);
    let root_area = root_area
        .titled("Hello World!", &font_large)
        .unwrap()
        .margin(0, 0, 0, 20);

    let (upper, lower) = root_area.split_vertically(512);

    let mut cc = ChartBuilder::on(&upper)
        .set_x_label_size(50)
        .set_y_label_size(60)
        .caption(
            "Sine and Cosine",
            &font_small
        )
        .build_ranged::<RangedCoordf32, RangedCoordf32, _, _>(-3.4f32..3.4f32, -1.2f32..1.2f32);

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()
        .unwrap();

    cc.draw_series(LineSeries::new(
        (0..12).map(|x| ((x - 6) as f32 / 2.0, ((x - 6) as f32 / 2.0).sin())),
        &RGBColor(255,0,0),
    ))
    .unwrap();
    cc.draw_series(LineSeries::new(
        (0..6800).map(|x| {
            (
                (x - 3400) as f32 / 1000.0,
                ((x - 3400) as f32 / 1000.0).cos(),
            )
        }),
        &RGBColor(0,0,255),
    ))
    .unwrap();

    cc.draw_series(PointSeries::<_, _, Cross<_>>::new(
        (0..6).map(|x| ((x - 3) as f32 / 1.0, ((x - 3) as f32 / 1.0).sin())),
        5,
        &RGBColor(0,0,0),
    ))
    .unwrap();

    let drawing_areas = lower.split_evenly((1, 2));

    for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .set_x_label_size(50)
            .set_y_label_size(60)
            .caption(
                format!("Chart {}", idx),
                &font_large
            )
            .build_ranged::<RangedCoordf32, RangedCoordf32, _, _>(0f32..11f32, 0f32..11f32);
        cc.configure_mesh().x_labels(5).y_labels(5).draw().unwrap();
    }

    root_area.close().unwrap();
}
