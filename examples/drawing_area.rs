use plotters::prelude::*;
fn main() {
    let mut backend = BitMapBackend::new("/tmp/plotter.png", (1024, 768));
    backend.open().unwrap();

    let area: DrawingArea<BitMapBackend, _> = backend.into();

    area.fill(&RGBColor(255, 255, 255)).unwrap();

    let area = area
        .titled(
            "Hello World",
            &Into::<FontDesc>::into("ArialMT").resize(80.0),
        )
        .unwrap();

    let (upper, lower) = area.split_vertically(256);
    upper.fill(&RGBColor(255, 0, 0)).unwrap();

    let path_color = Plattle9999::pick(15);

    let path = Path::new(
        vec![(0, 0), (50, 50), (70, 70), (30, 100), (0, 0)],
        &path_color,
    );

    for (a, idx) in lower.split_evenly((3, 3)).into_iter().zip(0..) {
        a.fill(&Plattle9999::pick(idx)).unwrap();
        a.draw(&path).unwrap();
    }

    area.close().unwrap();
}
