use plotters::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = SVGBackend::new("plotters-doc-data/3d-plot.svg", (600, 400)).into_drawing_area();
    area.fill(&WHITE)?;
    let x_axis = (-3.0..3.0).step(0.1);
    let z_axis = (-3.0..3.0).step(0.1);

    let mut chart = ChartBuilder::on(&area)
        .caption(format!("3D Plot Test"), ("sans", 20))
        .build_cartesian_3d(-3.0..3.0, -3.0..3.0, -3.0..3.0)?;

    chart.with_projection(|mut pb| {
        pb.yaw = 0.5;
        pb.scale = 0.9;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    let surface = SurfaceSeries::<f64, f64, f64>::new(
        x_axis.values(),
        z_axis.values(),
        |&x, &z| (x * x + z * z).cos(),
        &BLUE.mix(0.2),
    );

    chart
        .draw_series(surface)?
        .label("Surface")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .draw_series(LineSeries::new(
            (-100..100)
                .map(|y| y as f64 / 40.0)
                .map(|y| ((y * 10.0).sin(), y, (y * 10.0).cos())),
            &BLACK,
        ))?
        .label("Line")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}
