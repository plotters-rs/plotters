use plotters::prelude::*;

use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rand_xorshift::XorShiftRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<_> = {
        let norm_dist = Normal::new(500.0, 100.0).unwrap();
        let mut x_rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
        let x_iter = norm_dist.sample_iter(&mut x_rand);
        x_iter
            .filter(|x| *x < 1500.0)
            .take(100)
            .zip(0..)
            .map(|(x, b)| x + (b as f64).powf(1.2))
            .collect()
    };

    let root =
        BitMapBackend::new("plotters-doc-data/area-chart.png", (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .caption("Area Chart Demo", ("sans-serif", 40))
        .build_cartesian_2d(0..(data.len() - 1), 0.0..1500.0)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    chart.draw_series(
        AreaSeries::new(
            (0..).zip(data.iter()).map(|(x, y)| (x, *y)),
            0.0,
            &RED.mix(0.2),
        )
        .border_style(&RED),
    )?;

    Ok(())
}
#[test]
fn entry_point() {
    main().unwrap()
}
