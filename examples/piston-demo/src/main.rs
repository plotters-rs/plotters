use piston_window::{EventLoop, PistonWindow, WindowSettings};
use plotters::prelude::*;
use systemstat::platform::common::Platform;
use systemstat::System;

use std::collections::vec_deque::VecDeque;

const FPS: u32 = 10;
const LENGTH: u32 = 20;
const N_DATA_POINTS: usize = (FPS * LENGTH) as usize;
fn main() {
    let mut window: PistonWindow = WindowSettings::new("Real Time CPU Usage", [450, 300])
        .samples(4)
        .build()
        .unwrap();
    let sys = System::new();
    window.set_max_fps(FPS as u64);
    let mut load_measurement: Vec<_> = (0..FPS).map(|_| sys.cpu_load().unwrap()).collect();
    let mut epoch = 0;
    let mut data = vec![];
    while let Some(_) = draw_piston_window(&mut window, |b| {
        let cpu_loads = load_measurement[epoch % FPS as usize].done()?;

        let root = b.into_drawing_area();
        root.fill(&WHITE)?;

        if data.len() < cpu_loads.len() {
            for _ in data.len()..cpu_loads.len() {
                data.push(VecDeque::from(vec![0f32; N_DATA_POINTS + 1]));
            }
        }

        for (core_load, target) in cpu_loads.into_iter().zip(data.iter_mut()) {
            if target.len() == N_DATA_POINTS + 1 {
                target.pop_front();
            }
            target.push_back(1.0 - core_load.idle);
        }

        let mut cc = ChartBuilder::on(&root)
            .margin(10)
            .caption("Real Time CPU Usage", ("sans-serif", 30).into_font())
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_ranged(0..N_DATA_POINTS as u32, 0f32..1f32)?;

        cc.configure_mesh()
            .x_label_formatter(&|x| format!("{}", -(LENGTH as f32) + (*x as f32 / FPS as f32)))
            .y_label_formatter(&|y| format!("{}%", (*y * 100.0) as u32))
            .x_labels(15)
            .y_labels(5)
            .x_desc("Seconds")
            .y_desc("% Busy")
            .axis_desc_style(("sans-serif", 15).into_font())
            .draw()?;

        for (idx, data) in (0..).zip(data.iter()) {
            cc.draw_series(LineSeries::new(
                (0..).zip(data.iter()).map(|(a, b)| (a, *b)),
                &Palette99::pick(idx),
            ))?
            .label(format!("CPU {}", idx))
            .legend(move |(x, y)| {
                Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
            });
        }

        cc.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        load_measurement[epoch % FPS as usize] = sys.cpu_load()?;
        epoch += 1;
        Ok(())
    }) {}
}
