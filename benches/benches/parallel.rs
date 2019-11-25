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
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

fn draw_func_1x1_seq(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];
    c.bench_function("parallel::draw_func_1x1_seq", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            draw_plot(&root, 2.0);
        })
    });
}

fn draw_func_4x4(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_func_4x4");

    group.bench_function("parallel", |b| {
        let mut buffer = vec![0; (W * H * 3) as usize];
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            let areas = root.split_evenly((4, 4));
            areas.iter().for_each(|area| draw_plot(&area, 2.0));
        })
    });

    group.bench_function("blit", |b| {
        let mut buffer = vec![0; (W * H * 3) as usize];
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

fn draw_func_2x1(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_func_2x1");
    group.bench_function("blit", |b| {
        let mut buffer = vec![0; (W * H * 3) as usize];
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            let areas = root.split_evenly((2, 1));
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

    group.bench_function("inplace", |b| {
        let mut buffer = vec![0; (W * H * 3) as usize];
        b.iter(|| {
            let mut back = BitMapBackend::with_buffer(&mut buffer, (W, H));
            back.split(&[H / 2])
                .into_par_iter()
                .for_each(|b| draw_plot(&b.into_drawing_area(), 2.0));
        })
    });

    group.bench_function("sequential", |b| {
        let mut buffer = vec![0; (W * H * 3) as usize];
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            root.split_evenly((2, 1))
                .iter_mut()
                .for_each(|area| draw_plot(area, 2.0));
        })
    });
}

criterion_group! {
    name = parallel_group;
    config = Criterion::default().sample_size(10);
    targets =
        draw_func_1x1_seq,
        draw_func_4x4,
        draw_func_2x1,
}
