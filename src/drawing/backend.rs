use crate::style::{Color, FontDesc, FontError, Mixable};
use std::error::Error;

/// A coordiante in the image
pub type BackendCoord = (i32, i32);

/// The Error Type of a drawing backend
#[derive(Debug)]
pub enum DrawingErrorKind<E: Error> {
    /// A drawing backend error
    DrawingError(E),
    /// A font rendering error
    FontError(FontError),
}

impl<E: Error> std::fmt::Display for DrawingErrorKind<E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return match self {
            DrawingErrorKind::DrawingError(e) => write!(fmt, "Drawing backend error: {}", e),
            DrawingErrorKind::FontError(e) => write!(fmt, "Font loading error: {}", e),
        };
    }
}

impl<E: Error> Error for DrawingErrorKind<E> {}

/// The drawing context
pub trait DrawingBackend {
    /// The error reported by the backend
    type ErrorType: Error;

    /// Dimension
    fn get_size(&self) -> (u32, u32);

    /// Start drawing
    fn open(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Stop drawing
    fn close(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a pixel
    fn draw_pixel<C: Color>(
        &mut self,
        point: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a line
    fn draw_line<C: Color>(
        &mut self,
        mut from: BackendCoord,
        mut to: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let steep = (from.0 - to.0).abs() < (from.1 - to.1).abs();

        if steep {
            from = (from.1, from.0);
            to = (to.1, to.0);
        }

        let (from, to) = if from.0 > to.0 {
            (to, from)
        } else {
            (from, to)
        };

        let grad = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;

        let mut put_pixel = |(x, y): BackendCoord, b: f64| {
            if steep {
                return self.draw_pixel((y, x), &color.mix(b));
            } else {
                return self.draw_pixel((x, y), &color.mix(b));
            }
        };

        let mut y = from.1 as f64;

        for x in from.0..=to.0 {
            put_pixel((x, y as i32), 1.0 + y.floor() - y)?;
            put_pixel((x, y as i32 + 1), y - y.floor())?;

            y += grad;
        }

        return Ok(());
    }

    /// Draw a rectangle
    fn draw_rect<C: Color>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        color: &C,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (upper_left, bottom_right) = (
            (
                upper_left.0.min(bottom_right.0),
                upper_left.1.min(bottom_right.1),
            ),
            (
                upper_left.0.max(bottom_right.0),
                upper_left.1.max(bottom_right.1),
            ),
        );

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
            self.draw_line(
                (upper_left.0, upper_left.1),
                (upper_left.0, bottom_right.1),
                color,
            )?;
            self.draw_line(
                (upper_left.0, upper_left.1),
                (bottom_right.0, upper_left.1),
                color,
            )?;
            self.draw_line(
                (bottom_right.0, bottom_right.1),
                (upper_left.0, bottom_right.1),
                color,
            )?;
            self.draw_line(
                (bottom_right.0, bottom_right.1),
                (bottom_right.0, upper_left.1),
                color,
            )?;
        }
        return Ok(());
    }

    /// Draw a path
    fn draw_path<C: Color, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut begin: Option<BackendCoord> = None;
        for end in path.into_iter() {
            if let Some(begin) = begin {
                self.draw_line(begin, end, color)?;
            }
            begin = Some(end);
        }
        return Ok(());
    }

    /// Draw a circle
    fn draw_circle<C: Color>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        color: &C,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {

        //let range = ((radius + 3) / 4) as i32..=(2 * radius - radius / 4) as i32;
        let min = (radius as f64 * (1.0 - (2f64).sqrt() / 2.0)).ceil() as i32;
        let max = 2 * radius as i32 - min + 1;

        let range = min..max;

        let (up,down) = (range.start + center.1 - radius as i32, range.end + center.1 - radius as i32);

        for dy in range {
            let dy = dy - radius as i32;
            let y = center.1 + dy;

            let lx = (radius as f64 * radius as f64 - (dy as f64 * dy as f64).max(1e-5)).sqrt();

            let left = center.0 - lx.floor() as i32;
            let right = center.0 + lx.floor() as i32;

            let v = lx - lx.floor();
            
            let x = center.0 + dy;
            let top = center.1 - lx as i32;
            let bottom = center.1 + lx as i32;


            if fill {
                self.draw_line((left, y), (right, y), color)?;
                self.draw_line((x, top), (x, up), color)?;
                self.draw_line((x, down), (x, bottom), color)?;
            } else {
                self.draw_pixel((left, y), &color.mix(1.0-v))?;
                self.draw_pixel((right, y), &color.mix(1.0-v))?;
                
                self.draw_pixel((x, top), &color.mix(1.0-v))?;
                self.draw_pixel((x, bottom), &color.mix(1.0-v))?;
                
            }
            self.draw_pixel((left - 1, y), &color.mix(v))?;
            self.draw_pixel((right + 1, y), &color.mix(v))?;
            self.draw_pixel((x, top - 1), &color.mix(v))?;
            self.draw_pixel((x, bottom + 1), &color.mix(v))?;
        }

        return Ok(());
    }

    /// Draw a text
    fn draw_text<'a, C: Color>(
        &mut self,
        text: &str,
        font: &FontDesc<'a>,
        pos: BackendCoord,
        color: &C,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match font.draw(text, (pos.0, pos.1), |x, y, v| {
            self.draw_pixel((x as i32, y as i32), &color.mix(v as f64))
        }) {
            Ok(drawing_result) => drawing_result,
            Err(font_error) => Err(DrawingErrorKind::FontError(font_error)),
        }
    }
}
