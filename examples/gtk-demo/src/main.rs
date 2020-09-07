use std::env::args;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::Context;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;

fn build_ui(app: &gtk::Application) {
    drawable(app, 500, 500, |_, cr| {
        let root = CairoBackend::new(cr, (500, 500)).unwrap().into_drawing_area();

        root.fill(&WHITE).unwrap();
        let root = root.margin(25, 25, 25, 25);


        let mut chart = ChartBuilder::on(&root)
            .caption("This is a test", ("sans-serif", 20))
            .build_cartesian_3d(0.0..100.0, 0.0..100.0, 0.0..100.0)
            .unwrap();

        chart.with_projection(|mut p| { 
            p.scale = 0.9;
            p.into_matrix()
        });
        
        chart.configure_axes()
            .draw()
            .unwrap();

        chart.draw_series(
            SurfaceSeries::xoz(
                (0..100).map(|n| n as f64),
                (0..100).map(|n| n as f64),
                |x,z| ((x - z) / 10.0).sin() * 20.0 + 50.0,
            )
        ).unwrap();

        Inhibit(false)
    })
}

fn main() {
    let application = gtk::Application::new(
        Some("io.github.plotters-rs.plotters-gtk-test"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

pub fn drawable<F>(application: &gtk::Application, width: i32, height: i32, draw_fn: F)
where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
{
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = Box::new(DrawingArea::new)();

    drawing_area.connect_draw(draw_fn);

    window.set_default_size(width, height);

    window.add(&drawing_area);
    window.show_all();
}
