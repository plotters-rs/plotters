use crate::color::Color;
use crate::font::FontDesc;

/// A coordiante in the image
pub type Coord = (i32, i32);

/// The drawing context
pub trait DrawingBackend {
    /// The error reported by the backend
    type ErrorType;

    /// Dimension
    fn get_size(&self) -> (u32, u32);

    /// Start drawing
    fn open(&mut self) -> Result<(), Self::ErrorType>;

    /// Stop drawing
    fn close(&mut self) -> Result<(), Self::ErrorType>;

    /// Draw a pixel
    fn draw_pixel<C:Color>(&mut self, point: Coord, color : &C) -> Result<(), Self::ErrorType>;

    /// Draw a line
    fn draw_line<C:Color>(&mut self, mut from: Coord, mut to:Coord, color: &C) -> Result<(), Self::ErrorType> {
        let steep = (from.0 - to.0).abs() < (from.1 - to.1).abs();

        if steep {
            from = (from.1, from.0);
            to = (to.1, to.0);
        }

        let (from,to) = if from.0 > to.0 { (to, from) } else { (from, to) };

        let grad = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;

        let mut put_pixel = |(x,y):Coord, b:f64| {
            if steep {
                return self.draw_pixel((y,x),&color.mix(b));
            } else {
                return self.draw_pixel((x,y),&color.mix(b));
            }
        };

        let mut y = from.1 as f64;

        for x in from.0..=to.0 {
            put_pixel((x, y as i32), 1.0 + y.floor() - y)?;
            put_pixel((x, y as i32), y - y.floor())?;

            y += grad;
        }

        return Ok(());
    }

    /// Draw a rectangle
    fn draw_rect<C:Color>(&mut self, upper_left:Coord, bottom_right:Coord, color: &C, fill:bool) -> Result<(), Self::ErrorType> {
        if fill {
            if bottom_right.0 - upper_left.0 < bottom_right.1 - upper_left.1 {
                for x in upper_left.0..=bottom_right.0 {
                    self.draw_line((x, upper_left.1), (x, bottom_right.1), color)?;
                }
            } else {
                for y in upper_left.1..=bottom_right.1 {
                    self.draw_line((upper_left.0, y), (bottom_right.0, y), color)?;
                }
            }
        } else {
            self.draw_line((upper_left.0, upper_left.1), (upper_left.0, bottom_right.1), color)?;
            self.draw_line((upper_left.0, upper_left.1), (bottom_right.0, upper_left.1), color)?;
            self.draw_line((bottom_right.0, bottom_right.1), (upper_left.0, bottom_right.1), color)?;
            self.draw_line((bottom_right.0, bottom_right.1), (bottom_right.0, upper_left.1), color)?;
        }
        return Ok(());
    }

    /// Draw a path
    fn draw_path<C:Color, I:IntoIterator<Item=Coord> >(&mut self, path:I, color: &C) -> Result<(), Self::ErrorType> {
        let mut begin:Option<Coord> = None;
        for end in path.into_iter() {
            if let Some(begin) = begin {
                self.draw_line(begin, end, color)?;
            }
            begin = Some(end);
        }
        return Ok(());
    }

    /// Draw a circle
    fn draw_circle<C:Color>(&mut self, center:Coord, radius:u32, color: &C, fill:bool) -> Result<(), Self::ErrorType> {
        let range = if fill {
            0..=2*radius as i32
        } else {
            ((radius + 3)/4) as i32..=(2*radius-radius/4) as i32
        };
        for dy in range {
            let dy = dy - radius as i32;
            let y = center.1 + dy;

            let lx = (radius as f64 * radius as f64 - dy as f64* dy as f64).sqrt();

            let left = center.0 - lx as i32;
            let right = center.0 + lx as i32;

            if fill {
                self.draw_line((left,y),(right,y),color)?;
            } else {
                self.draw_pixel((left,y), color)?;
                self.draw_pixel((right,y), color)?;
                
                let x = center.0 + dy;
                let left = center.1 - lx as i32;
                let right = center.1 + lx as i32;

                self.draw_pixel((x,left), color)?;
                self.draw_pixel((x,right), color)?;
            }
        }

        return Ok(());
    }

    /// Draw a text
    fn draw_text<'a, C:Color>(&mut self, text:&str, font: &FontDesc<'a>, pos: Coord, color: &C) -> Result<(), Self::ErrorType> {
        font.draw(text, (pos.0 as u32, pos.1 as u32), |x,y,v| {
            self.draw_pixel((x as i32,y as i32), &color.mix(v as f64)).ok();
        });
        Ok(())
    }
}
