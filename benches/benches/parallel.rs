use criterion::{criterion_group, Criterion, ParameterizedBenchmark};

use plotters::coord::Shift;
use plotters::prelude::*;
use rayon::prelude::*;

const SIZES: &'static [u32] = &[100, 400, 800, 1000, 2000];

fn draw_plot(root: &DrawingArea<BitMapBackend, Shift>, pow: f64) {
    let mut chart = ChartBuilder::on(root)
        .caption(format!("y = x^{}", pow), ("Arial", 30))
        .build_cartesian_2d(-1.0..1.0, -1.0..1.0)
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
    c.bench(
        "draw_func_1x1",
        ParameterizedBenchmark::new(
            "sequential",
            |b, &&s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                b.iter(|| {
                    let root = BitMapBackend::with_buffer(&mut buffer, (s, s)).into_drawing_area();
                    root.fill(&WHITE).unwrap();
                    draw_plot(&root, 2.0);
                })
            },
            SIZES.clone(),
        ),
    );
}

fn draw_func_4x4(c: &mut Criterion) {
    c.bench(
        "draw_func_4x4",
        ParameterizedBenchmark::new(
            "sequential",
            |b, &&s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                b.iter(|| {
                    let root = BitMapBackend::with_buffer(&mut buffer, (s, s)).into_drawing_area();
                    let areas = root.split_evenly((4, 4));
                    areas.iter().for_each(|area| draw_plot(&area, 2.0));
                })
            },
            SIZES.clone(),
        )
        .with_function("blit", |b, &&s| {
            let mut buffer = vec![0; (s * s * 3) as usize];
            let mut element_buffer = vec![vec![0; (s * s / 4 * 3) as usize]; 4];
            b.iter(|| {
                let root = BitMapBackend::with_buffer(&mut buffer, (s, s)).into_drawing_area();
                let areas = root.split_evenly((4, 4));
                let elements: Vec<_> = element_buffer
                    .par_iter_mut()
                    .map(|b| {
                        let mut e = BitMapElement::with_mut((0, 0), (s / 2, s / 2), b).unwrap();
                        draw_plot(&e.as_bitmap_backend().into_drawing_area(), 2.0);
                        e
                    })
                    .collect();

                areas
                    .into_iter()
                    .zip(elements.into_iter())
                    .for_each(|(a, e)| a.draw(&e).unwrap());
            })
        })
        .with_function("inplace-blit", |b, &&s| {
            let mut buffer = vec![0; (s * s * 3) as usize];
            let mut element_buffer = vec![vec![vec![0; (s * s / 4 * 3) as usize]; 2]; 2];
            b.iter(|| {
                let mut back = BitMapBackend::with_buffer(&mut buffer, (s, s));
                back.split(&[s / 2])
                    .into_iter()
                    .zip(element_buffer.iter_mut())
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .for_each(|(back, buffer)| {
                        let root = back.into_drawing_area();
                        let areas = root.split_evenly((1, 2));

                        let elements: Vec<_> = buffer
                            .par_iter_mut()
                            .map(|b| {
                                let mut e =
                                    BitMapElement::with_mut((0, 0), (s / 2, s / 2), b).unwrap();
                                draw_plot(&e.as_bitmap_backend().into_drawing_area(), 2.0);
                                e
                            })
                            .collect();

                        areas
                            .into_iter()
                            .zip(elements.into_iter())
                            .for_each(|(a, e)| a.draw(&e).unwrap())
                    });
            })
        }),
    );
}

fn draw_func_2x1(c: &mut Criterion) {
    c.bench(
        "draw_func_2x1",
        ParameterizedBenchmark::new(
            "blit",
            |b, &&s| {
                let mut buffer = vec![0; (s * s * 3) as usize];
                let mut element_buffer = vec![vec![0; (s * s / 2 * 3) as usize]; 2];
                b.iter(|| {
                    let root = BitMapBackend::with_buffer(&mut buffer, (s, s)).into_drawing_area();
                    let areas = root.split_evenly((2, 1));
                    let elements: Vec<_> = element_buffer
                        .par_iter_mut()
                        .map(|buf| {
                            let mut element =
                                BitMapElement::with_mut((0, 0), (s, s / 2), buf).unwrap();
                            draw_plot(&element.as_bitmap_backend().into_drawing_area(), 2.0);
                            element
                        })
                        .collect();

                    areas
                        .into_iter()
                        .zip(elements.into_iter())
                        .for_each(|(a, e)| a.draw(&e).unwrap());
                })
            },
            SIZES.clone(),
        )
        .with_function("inplace", |b, &&s| {
            let mut buffer = vec![0; (s * s * 3) as usize];
            b.iter(|| {
                let mut back = BitMapBackend::with_buffer(&mut buffer, (s, s));
                back.split(&[s / 2])
                    .into_par_iter()
                    .for_each(|b| draw_plot(&b.into_drawing_area(), 2.0));
            })
        })
        .with_function("sequential", |b, &&s| {
            let mut buffer = vec![0; (s * s * 3) as usize];
            b.iter(|| {
                let root = BitMapBackend::with_buffer(&mut buffer, (s, s)).into_drawing_area();
                root.split_evenly((2, 1))
                    .iter_mut()
                    .for_each(|area| draw_plot(area, 2.0));
            })
        }),
    );
}

criterion_group! {
    name = parallel_group;
    config = Criterion::default().sample_size(10);
    targets =
        draw_func_1x1_seq,
        draw_func_4x4,
        draw_func_2x1,
}
