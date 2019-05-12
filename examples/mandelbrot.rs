use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = BitMapBackend::new("examples/outputs/mandelbrot.png", (800, 600));

    backend.open()?;
    let root: DrawingArea<_, _> = backend.into();
    root.fill(&White)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_ranged(-2.1f64..0.6f64, -1.2f64..1.2f64);

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    let range = plotting_area.get_pixel_range();
    let (pw, ph) = (range.0.end - range.0.start, range.1.end - range.1.start);

    for (x,y,c) in MandelbrotSet::new(-2.1..0.6, -1.2..1.2, (pw as usize, ph as usize), 200) {
        if c != 200 {
            plotting_area.draw_pixel((x,y), &Palette99::pick(c/10).mix((c%10) as f64 / 10.0))?;
        } else {
            plotting_area.draw_pixel((x,y), &Black)?;
        }
    }

    root.close()?;
    return Ok(());
}

use std::ops::Range;
struct MandelbrotSet {
    start: (f64, f64),
    step: (f64, f64),
    size: (usize, usize),
    this: (usize, usize),
    iter: usize,
}

impl MandelbrotSet {
    fn new(x: Range<f64>, y: Range<f64>, (nx, ny): (usize, usize), niter: usize) -> Self {
        return MandelbrotSet {
            start: (x.start, y.start),
            step: ((x.end - x.start) / nx as f64, (y.end - y.start) / ny as f64),
            size: (nx, ny),
            this: (0, 0),
            iter: niter,
        };
    }
}

impl Iterator for MandelbrotSet {
    type Item = (f64, f64, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.this.0 < self.size.0 && self.this.1 < self.size.1 {
            let (c_r, c_c) = (
                self.start.0 + self.this.0 as f64 * self.step.0,
                self.start.1 + self.this.1 as f64 * self.step.1,
            );
            let (mut z_r, mut z_c) = (0.0, 0.0);
            let mut n_iter = 0;
            for _ in 0..self.iter {
                n_iter += 1;

                let nz_r = z_r * z_r - z_c * z_c + c_r;
                let nz_c = 2.0 * z_c * z_r + c_c;

                z_r = nz_r;
                z_c = nz_c;

                if z_r * z_r + z_c * z_c > 1e10 {
                    break;
                }
            }

            self.this.0 += 1;

            if self.this.0 == self.size.0 {
                self.this.0 = 0;
                self.this.1 += 1;
            }

            return Some((c_r, c_c, n_iter));
        }
        return None;
    }
}

