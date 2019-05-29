use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::Color;
use image::{ImageError, Rgb, RgbImage};

/// The backend that drawing a bitmap
pub struct BitMapBackend<'a> {
    /// The path to the image
    path: &'a str,
    /// The image object
    img: RgbImage,
}

impl<'a> BitMapBackend<'a> {
    /// Create a new bitmap backend
    pub fn new(path: &'a str, dimension: (u32, u32)) -> Self {
        Self {
            path,
            img: RgbImage::new(dimension.0, dimension.1),
        }
    }
}

impl<'a> DrawingBackend for BitMapBackend<'a> {
    type ErrorType = ImageError;

    fn get_size(&self) -> (u32, u32) {
        (self.img.width(), self.img.height())
    }

    fn open(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        self.img
            .save(&self.path)
            .map_err(|x| DrawingErrorKind::DrawingError(ImageError::IoError(x)))
    }

    fn draw_pixel<C: Color>(
        &mut self,
        point: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<ImageError>> {
        if point.0 as u32 >= self.img.width()
            || point.0 < 0
            || point.1 as u32 >= self.img.height()
            || point.1 < 0
        {
            return Ok(());
        }

        let alpha = color.alpha();
        let rgb = color.rgb();

        if alpha >= 1.0 {
            self.img.put_pixel(
                point.0 as u32,
                point.1 as u32,
                Rgb {
                    data: [rgb.0, rgb.1, rgb.2],
                },
            );
        } else {
            let pixel = self.img.get_pixel_mut(point.0 as u32, point.1 as u32);

            let new_color = [rgb.0, rgb.1, rgb.2];

            pixel
                .data
                .iter_mut()
                .zip(&new_color)
                .for_each(|(old, new)| {
                    *old = (f64::from(*old) * (1.0 - alpha) + f64::from(*new) * alpha).min(255.0)
                        as u8;
                });
        }
        Ok(())
    }
}
