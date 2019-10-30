use plotters::prelude::*;

use rand::thread_rng;
use rand_distr::{Distribution, Normal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        BitMapBackend::<image::Rgb<u8>>::new("plotters-doc-data/normal-dist.png", (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let sd = 0.13;

    let random_points: Vec<(f64, f64)> = {
        let norm_dist = Normal::new(0.5, sd).unwrap();
        let (mut x_rand, mut y_rand) = (thread_rng(), thread_rng());
        let x_iter = norm_dist.sample_iter(&mut x_rand);
        let y_iter = norm_dist.sample_iter(&mut y_rand);
        x_iter.zip(y_iter).take(5000).collect()
    };

    let areas = root.split_by_breakpoints([944], [80]);

    let mut x_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(40)
        .build_ranged(0u32..100u32, 0f64..0.5f64)?;
    let mut y_hist_ctx = ChartBuilder::on(&areas[3])
        .x_label_area_size(40)
        .build_ranged(0f64..0.5f64, 0..100u32)?;
    let mut scatter_ctx = ChartBuilder::on(&areas[2])
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_ranged(0f64..1f64, 0f64..1f64)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(
        random_points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
    )?;
    let x_hist = Histogram::vertical(&x_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(
            random_points
                .iter()
                .map(|(x, _)| ((x * 100.0) as u32, 0.002)),
        );
    let y_hist = Histogram::horizontal(&y_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(
            random_points
                .iter()
                .map(|(_, y)| ((y * 100.0) as u32, 0.002)),
        );
    x_hist_ctx.draw_series(x_hist)?;
    y_hist_ctx.draw_series(y_hist)?;

    Ok(())
}
