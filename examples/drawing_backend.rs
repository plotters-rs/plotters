use plotters::drawing::{backend::DrawingBackend, BitMapBackend};
use plotters::style::{FontDesc, Mixable, Plattle, Plattle9999, RGBColor};
fn main() {
    let mut backend = BitMapBackend::new("/tmp/plotter.png", (1024, 768));
    backend.open().unwrap();
    backend
        .draw_rect((0, 0), (1024, 768), &RGBColor(255, 255, 255), true)
        .unwrap();
    backend
        .draw_line((0, 0), (100, 100), &Plattle9999::pick(5))
        .unwrap();
    backend
        .draw_circle((500, 500), 300, &Plattle9999::pick(0).mix(0.5), true)
        .unwrap();
    backend
        .draw_circle((600, 600), 300, &Plattle9999::pick(1).mix(0.5), true)
        .unwrap();
    backend
        .draw_rect((100, 100), (500, 500), &Plattle9999::pick(2).mix(0.7), true)
        .unwrap();
    backend
        .draw_text(
            "abcdefghijklmnopqrstuvwxyz!@#%",
            &FontDesc::new("ArialMT", 40.0),
            (300, 500),
            &Plattle9999::pick(3),
        )
        .unwrap();
    backend.close().unwrap();
}
