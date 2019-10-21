use criterion::{criterion_group, Criterion};
use plotters::prelude::*;

const W: u32 = 1000;
const H: u32 = 1000;

fn draw_pixel(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    c.bench_function("rasterizer::draw_pixel", |b| {
        b.iter(|| {
            let mut root = BitMapBackend::with_buffer(&mut buffer, (W, H));
            for x in 0..W / 10 {
                for y in 0..H / 10 {
                    root.draw_pixel((x as i32, y as i32), &RGBColor(255, 0, 234).to_rgba())
                        .unwrap();
                }
            }
        })
    });
}

fn draw_line(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    c.bench_function("rasterizer::draw_line", |b| {
        b.iter(|| {
            let mut root = BitMapBackend::with_buffer(&mut buffer, (W, H));
            for y in 0..10 {
                root.draw_line(
                    (0, 0),
                    ((W / 2) as i32, (y * 100) as i32),
                    &RGBColor(255, 0, 234).to_rgba(),
                )
                .unwrap();
            }
        })
    });
}

fn fill_background(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    c.bench_function("rasterizer::fill_background", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            root.fill(&WHITE).unwrap();
        })
    });
}

fn fill_circle(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    c.bench_function("rasterizer::fill_circle", |b| {
        b.iter(|| {
            let mut root = BitMapBackend::with_buffer(&mut buffer, (W, H));
            root.draw_circle((W as i32 / 2, H as i32 / 2), W / 2, &WHITE.to_rgba(), true)
                .unwrap();
        })
    });
}

fn fill_background_red(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    c.bench_function("rasterizer::fill_background_red", |b| {
        b.iter(|| {
            let root = BitMapBackend::with_buffer(&mut buffer, (W, H)).into_drawing_area();
            root.fill(&RED).unwrap();
        })
    });
}

fn fill_hexagon(c: &mut Criterion) {
    let mut buffer = vec![0; (W * H * 3) as usize];

    let mut vert = vec![];

    for i in 0..6 {
        let x = (W as f64 / 5.0 * (std::f64::consts::PI * i as f64 / 3.0).cos()).ceil() as i32
            + W as i32 / 2;
        let y = (W as f64 / 5.0 * (std::f64::consts::PI * i as f64 / 3.0).sin()).ceil() as i32
            + W as i32 / 2;
        vert.push((x, y));
    }

    c.bench_function("rasterizer::fill_hexagon", |b| {
        b.iter(|| {
            let mut root = BitMapBackend::with_buffer(&mut buffer, (W, H));
            root.fill_polygon(vert.clone(), &RED).unwrap();
        })
    });
}

criterion_group! {
    name = rasterizer_group;
    config = Criterion::default();
    targets = 
        draw_pixel, 
        draw_line, 
        fill_background, 
        fill_circle, 
        fill_background_red, 
        fill_hexagon
}
