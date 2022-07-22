use plotters::prelude::*;

fn snowflake_iter(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut ret = vec![];
    for i in 0..points.len() {
        let (start, end) = (points[i], points[(i + 1) % points.len()]);
        let t = ((end.0 - start.0) / 3.0, (end.1 - start.1) / 3.0);
        let s = (
            t.0 * 0.5 - t.1 * (0.75f64).sqrt(),
            t.1 * 0.5 + (0.75f64).sqrt() * t.0,
        );
        ret.push(start);
        ret.push((start.0 + t.0, start.1 + t.1));
        ret.push((start.0 + t.0 + s.0, start.1 + t.1 + s.1));
        ret.push((start.0 + t.0 * 2.0, start.1 + t.1 * 2.0));
    }
    ret
}

const OUT_FILE_NAME: &'static str = "plotters-doc-data/snowflake.png";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Koch's Snowflake", ("sans-serif", 50))
        .build_cartesian_2d(-2.0..2.0, -1.5..1.5)?;

    let mut snowflake_vertices = {
        let mut current: Vec<(f64, f64)> = vec![
            (0.0, 1.0),
            ((3.0f64).sqrt() / 2.0, -0.5),
            (-(3.0f64).sqrt() / 2.0, -0.5),
        ];
        for _ in 0..6 {
            current = snowflake_iter(&current[..]);
        }
        current
    };

    chart.draw_series(std::iter::once(Polygon::new(
        snowflake_vertices.clone(),
        &RED.mix(0.2),
    )))?;
    snowflake_vertices.push(snowflake_vertices[0]);
    chart.draw_series(std::iter::once(PathElement::new(snowflake_vertices, &RED)))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
