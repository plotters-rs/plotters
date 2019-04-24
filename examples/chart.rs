use plotters::ChartBuilder;
use plotters::drawing::{BitMapBackend, DrawingArea};
use plotters::drawing::backend::DrawingBackend;
use plotters::style::{TextStyle, FontDesc, RGBColor, ShapeStyle};
use plotters::drawing::coord::RangedCoordf32;
fn main() {
    let mut img = BitMapBackend::new("/tmp/plotter.png", (1024, 768));
    
    img.open().unwrap();

    let root_area:DrawingArea<_, _> = img.into();

    root_area.fill(&RGBColor(255,255,255)).unwrap();

    let caption_font = FontDesc::new("ArialMT", 60.0);
    let chart_font = FontDesc::new("ArialMT", 20.0);
    let black = RGBColor(0,0,0);
    let caption_style = TextStyle {
        font: &caption_font,
        color: &black
    };
    let root_area = root_area.titled("Demo Title", caption_style).unwrap();

    let (upper, lower) = root_area.split_vertically(512);

    let mut cc = ChartBuilder::on(&upper)
        .set_x_label_size(50)
        .set_y_label_size(60)
        .build_ranged::<RangedCoordf32, RangedCoordf32, _, _>(-3.4f32..3.4f32, -1.2f32..1.2f32);

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .x_label_formatter(Box::new(|v| format!("{:.1}", v)))
        .y_label_formatter(Box::new(|v| format!("{:.1}", v)))
        .draw().unwrap();

    let red = RGBColor(255,0,0);
    let blue = RGBColor(0,0,255);
    
    cc.draw_series(plotters::series::LineSeries::new((0..6800).map(|x| ((x-3400) as f32/1000.0, ((x-3400) as f32 / 1000.0).sin())), &ShapeStyle{color:&red})).unwrap();
    cc.draw_series(plotters::series::LineSeries::new((0..6800).map(|x| ((x-3400) as f32/1000.0, ((x-3400) as f32 / 1000.0).cos())), &ShapeStyle{color:&blue})).unwrap();
    
    cc.draw_series(plotters::series::PointSeries::<_,_,plotters::element::Cross<_>>::new((0..6).map(|x| ((x-3) as f32/1.0, ((x-3) as f32 / 1.0).sin())), 5, &ShapeStyle{color:&black})).unwrap();
    
    let drawing_areas = lower.split_evenly((1,2));

    for (drawing_area,idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .set_x_label_size(50)
            .set_y_label_size(60)
            .caption(format!("Chart {}", idx), TextStyle { font:&chart_font, color:&black })
            .build_ranged::<RangedCoordf32, RangedCoordf32, _, _>(0f32..11f32, 0f32..11f32);
        cc.configure_mesh().draw().unwrap();
    }

    root_area.close().unwrap();
}
