use criterion::{criterion_group, Criterion};

use plotters::coord::Shift;
use plotters::prelude::*;
use rayon::prelude::*;

const W: u32 = 1000;
const H: u32 = 1000;

fn draw_plot(root: &DrawingArea<BitMapBackend, Shift>, pow: f64) {
    let mut chart = ChartBuilder::on(root)
        .caption(format!("y = x^{}", pow), ("Arial", 30))
        .build_ranged(-1.0..1.0, -1.0..1.0)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (-50..=50)
                .map(|x| x as f64 / 50.0)
                .map(|x| (x, x.powf(pow))),
            &RED,
        ))
        .unwrap()
        .label(format!("y = x^{}", pow))
        .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RED));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn draw_func_1x1_seq(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];
    c.bench_function("draw_func_1x1_seq", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            draw_plot(&root, 2.0);
        })
    });
}

fn draw_func_4x4_seq(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];
    c.bench_function("draw_func_4x4_seq", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            let areas = root.split_evenly((4, 4));
            areas.iter().for_each(|area| draw_plot(&area, 2.0));
        })
    });
}

fn draw_func_4x4_parallel_and_blit(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];
    c.bench_function("draw_func_4x4_parallel_and_blit", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            let areas = root.split_evenly((4, 4));
            let mut elements: Vec<_> = areas
                .iter()
                .map(|area| area.dim_in_pixel())
                .map(|size| BitMapElement::new((0, 0), size))
                .collect();

            elements
                .par_iter_mut()
                .for_each(|e| draw_plot(&e.as_bitmap_backend().into_drawing_area(), 2.0));

            areas
                .into_iter()
                .zip(elements.into_iter())
                .for_each(|(a, e)| a.draw(&e).unwrap());
        })
    });
}

criterion_group! {
    name = parallel_group;
    config = Criterion::default().sample_size(10);
    targets = draw_func_1x1_seq, draw_func_4x4_seq, draw_func_4x4_parallel_and_blit
}
