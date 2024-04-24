use plotters::prelude::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("test.bmp", (1000, 1000)).into_drawing_area();
    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(0.0..1000.0, 0.0..1000.0)?;
    let pahts = vec![
        PathElement::new(
            vec![(336.0, 614.0), (339.0, 674.0),(341.0,714.0)],
            ShapeStyle::from(RED).stroke_width(2),
        ),
        PathElement::new(
            vec![(100.0, 100.0), (150.0, 150.0),(200.0,100.0)],
            ShapeStyle::from(BLUE).stroke_width(2),
        ),
        PathElement::new(
            vec![(400.0, 400.0), (400.0, 450.0),(400.0,500.0)],
            ShapeStyle::from(GREEN).stroke_width(5),
        ),
        PathElement::new(
            vec![(900.0, 410.0), (600.0, 400.0),(900.0,500.0), (1000.0, 1000.0)],
            ShapeStyle::from(YELLOW).stroke_width(10),
        ),
    ];
    chart.draw_series(pahts.into_iter())?;
    Ok(())
}