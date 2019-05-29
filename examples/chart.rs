use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = BitMapBackend::new("examples/outputs/sample.png", (1024, 768));

    let root_area: DrawingArea<_, _> = img.into();

    root_area.fill(&RGBColor(255, 255, 255))?;

    let font: FontDesc = "Arial".into();
    let font_large = &font.resize(60.0);
    let font_small = &font.resize(40.0);
    let root_area = root_area
        .titled("Image Title", &font_large)?
        .margin(0, 0, 0, 20);

    let (upper, lower) = root_area.split_vertically(512);

    let mut cc = ChartBuilder::on(&upper)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .caption("Sine and Cosine", &font_small)
        .build_ranged(-3.4f32..3.4f32, -1.2f32..1.2f32);

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_x_mesh()
        .disable_y_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    /*cc.define_series_label_area(
        (720, 130),
        (240, 100),
        Into::<ShapeStyle>::into(&RGBColor(255, 255, 255).mix(0.7)).filled(),
    )?;*/

    cc.draw_series(LineSeries::new(
        (0..12).map(|x| ((x - 6) as f32 / 2.0, ((x - 6) as f32 / 2.0).sin())),
        &RGBColor(255, 0, 0),
    ))?;

    cc.draw_series(LineSeries::new(
        (0..6800).map(|x| {
            (
                (x - 3400) as f32 / 1000.0,
                ((x - 3400) as f32 / 1000.0).cos(),
            )
        }),
        &RGBColor(0, 0, 255),
    ))?;

    // It's possible to use a existing pointing element
    /*
     cc.draw_series(PointSeries::<_, _, Circle<_>>::new(
        (0..6).map(|x| ((x - 3) as f32 / 1.0, ((x - 3) as f32 / 1.0).sin())),
        5,
        Into::<ShapeStyle>::into(&RGBColor(255,0,0)).filled(),
    ))?;*/

    let point_font = font_small.resize(15.0);
    // Otherwise you can use a function to construct your pointing element yourself
    cc.draw_series(PointSeries::of_element(
        (0..6).map(|x| ((x - 3) as f32 / 1.0, ((x - 3) as f32 / 1.0).sin())),
        5,
        Into::<ShapeStyle>::into(&RGBColor(255, 0, 0)).filled(),
        &|coord, size, style| {
            return EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + OwnedText::new(format!("{:?}", coord), (0, 15), &point_font);
        },
    ))?;

    let drawing_areas = lower.split_evenly((1, 2));

    for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .caption(format!("y = x^{}", 1 + 2 * idx), &font_small)
            .build_ranged(-1f32..1f32, -1f32..1f32);
        cc.configure_mesh().x_labels(5).y_labels(3).draw()?;

        cc.draw_series(LineSeries::new(
            (-100..100).map(|x| {
                (
                    x as f32 / 100.0,
                    (x as f32 / 100.0).powf(idx as f32 * 2.0 + 1.0),
                )
            }),
            &RGBColor(0, 0, 255),
        ))?;
    }

    root_area.present()?;

    return Ok(());
}
