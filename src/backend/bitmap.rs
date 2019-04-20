use image::{RgbImage, Rgb, ImageError};
use crate::drawing::{DrawingBackend, Coord};
use crate::color::Color;

pub struct BitMapBackend<'a> {
    path: &'a str,
    img : RgbImage,
}

impl <'a> BitMapBackend<'a> {
    pub fn new(path: &'a str, dimension: (u32,u32)) -> Self {
        return Self {
            path,
            img : RgbImage::new(dimension.0, dimension.1)
        };
    }
}

impl <'a> DrawingBackend for BitMapBackend<'a> {
    type ErrorType = ImageError;

    fn get_size(&self) -> (u32, u32) {
        return (self.img.width(), self.img.height());
    }

    fn open(&mut self) -> Result<(), ImageError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), ImageError> {
        self.img.save(&self.path).map_err(|x| ImageError::IoError(x))
    }

    fn draw_pixel<C:Color>(&mut self, point:Coord, color: &C) -> Result<(), ImageError> {
        if point.0 as u32 >= self.img.width()  || point.0 < 0 ||
           point.1 as u32 >= self.img.height() || point.1 < 0 {
            return Ok(());
        }

        let alpha = color.alpha();
        let rgb = color.rgb();

        if alpha == 1.0 {
            self.img.put_pixel(point.0 as u32, point.1 as u32, Rgb {
                data: [rgb.0, rgb.1, rgb.2]
            }); 
        } else {
            let pixel = self.img.get_pixel_mut(point.0 as u32, point.1 as u32);

            let new_color = [rgb.0, rgb.1, rgb.2];

            pixel.data.iter_mut().zip(&new_color).for_each(|(old, new)| {
                *old = (*old as f64 * (1.0 - alpha) + *new as f64 * alpha) as u8;
            });
        }
        return Ok(());
    }
}
