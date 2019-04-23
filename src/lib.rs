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
    use crate::color::{Color, RGBColor, PlattleColor};
    use crate::plattle::Plattle9999;
    use crate::drawing::DrawingBackend;
    use crate::font::FontDesc;
    use crate::region::{DrawingRegion, Splitable};
    use crate::element::*;

    #[test]
    fn it_works() {
        let mut b = BitMapBackend::new("/tmp/test.png", (1024, 1024));
        
        let color = RGBColor(255,0,0);
        b.open().unwrap();
        b.draw_rect((0,0), (1024,1024), &RGBColor(255,255,255), true).unwrap();

        let font = FontDesc::new("FandolHei", 10.0);

        /*b.draw_path(vec![(0,0), (100,100), (200, 400), (250, 625), (0,0) ].into_iter(), &color).unwrap();
        b.draw_circle((500,500), 200, &color.mix(0.5), true).unwrap();
        b.draw_circle((600,600), 200, &RGBColor(0,255,0).mix(0.5), true).unwrap();
        b.draw_text("Hello World", &font, (400, 400), &RGBColor(0,255, 255)).ok();
        b.draw_rect((400,400), (430,430), &RGBColor(255,255,255), true).ok();*/
        
        
        let dr1:DrawingRegion<_,_> = b.into();
        let dr = dr1.titled("Hello World!", &FontDesc::new("ArialMT", 100.0), RGBColor(0,0,0)).unwrap();
        let mut regions = dr.split_m_n((3,1));
        regions[0].split_m_n((1,3)).iter().zip(0..).for_each(|(r,x)| {
            r.fill(PlattleColor::<Plattle9999>::pick(1000 - x as usize));
            let text = format!("This is block upper {}!", x);
            let element = Text::new(&text, &font, (0, 0), PlattleColor::<Plattle9999>::pick(x as usize));
            r.draw(&element);
        });

        regions[1].split_m_n((1,2)).iter().zip(3..).for_each(|(r,x)| {
            r.fill(PlattleColor::<Plattle9999>::pick(1000 - x as usize));
            let text = format!("This is bottom block {}!", x);
            let element = Text::new(&text, &font, (0, 0), PlattleColor::<Plattle9999>::pick(x as usize));
            r.draw(&element);
        });

        let mut bottom_region = regions[2].titled("This is the bottom region", &FontDesc::new("ArialMT", 80.0), RGBColor(0,0,0)).unwrap();

        //println!("{:?}", drawing_region.size_in_pixels());
       /* let (w, h) = plot.size_in_pixels();
        let mut cmf_y = |y:&f32| {
            return h - (*y * h as f32 / 10.0) as u32;
        };

        let kps:Vec<_> = (0..10).map(|x| x as f32).collect();
        let grid = crate::element::Grid::new(crate::element::GridDirection::Horizontal, &kps, (0,w), cmf_y, RGBColor(0,0,0), 1);
        plot.draw(&grid);
        
        let kps:Vec<_> = (0..20).map(|x| x as f32 / 2.0).collect();
        let grid_2 = crate::element::Grid::new(crate::element::GridDirection::Horizontal, &kps, (0,w), cmf_y, RGBColor(0,0,0).mix(0.5), 1);
        plot.draw(&grid_2);

        for ((_,h),_,v) in grid.get_grid_lines() {
            let text = format!("{:.3}", v);
            let element = Text::new(&text, &font, (0, h) ,RGBColor(0,0,0));
            left.draw(&element);
        }*/
        let plot = crate::plot::Plot::<_, crate::plot::RangedCoordF32<crate::plot::Linear>, 
        crate::plot::RangedCoordF32<crate::plot::Linear>>::new(bottom_region, 
                (0f32..10f32).into(), 
                (0f32..10f32).into(), 50, 50);

        


        //regions[7].split((3,3)).iter().for_each(|r| {r.draw(&TestElem("A".to_string(), &font));});
        dr.close();
    }
}
