mod drawing;
pub mod font;
pub mod backend;
pub mod color;
pub mod plattle;
pub mod region;
pub mod element;
pub mod plot;




#[cfg(test)]
mod tests {
    use crate::backend::BitMapBackend;
    use crate::color::{Color, RGBColor};
    use crate::drawing::DrawingBackend;
    use crate::font::FontDesc;
    use crate::region::{DrawingRegion, Splitable};
    use crate::element::*;

    #[test]
    fn it_works() {
        let mut b = BitMapBackend::new("/tmp/test.png", (1024, 768));
        
        let color = RGBColor(255,0,0);
        b.open().unwrap();
        b.draw_rect((0,0), (1024,768), &RGBColor(255,255,255), true).unwrap();

        let font = FontDesc::new("FandolHei", 30.0);

        b.draw_path(vec![(0,0), (100,100), (200, 400), (250, 625), (0,0) ].into_iter(), &color).unwrap();
        b.draw_circle((500,500), 200, &color.mix(0.5), true).unwrap();
        b.draw_circle((600,600), 200, &RGBColor(0,255,0).mix(0.5), true).unwrap();
        b.draw_text("Hello World", &font, (400, 400), &RGBColor(0,255, 255)).ok();
        b.draw_rect((400,400), (430,430), &RGBColor(255,255,255), true).ok();
        
        
        let dr:DrawingRegion<_,_> = b.into();
        let mut regions = dr.split((2,1));
        regions[0].split((1,3)).iter().zip(0..).for_each(|(r,x)| {
            let text = format!("This is block upper {}!", x);
            let element = Text::new(&text, &font, (0, 0), RGBColor(255,0,0));
            r.draw(&element);
        });

        regions[1].split((1,2)).iter().zip(0..).for_each(|(r,x)| {
            let text = format!("This is bottom block {}!", x);
            let element = Text::new(&text, &font, (0, 0), RGBColor(255,0,0));
            r.draw(&element);
        });
        //regions[7].split((3,3)).iter().for_each(|r| {r.draw(&TestElem("A".to_string(), &font));});
        dr.close();
    }
}
