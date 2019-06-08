use crate::style::{Color, FontDesc, FontError, RGBAColor};
use std::error::Error;

/// A coordiante in the image
pub type BackendCoord = (i32, i32);

/// The error produced by a drawing backend
#[derive(Debug)]
pub enum DrawingErrorKind<E: Error + Send + Sync> {
    /// A drawing backend error
    DrawingError(E),
    /// A font rendering error
    FontError(FontError),
}

impl<E: Error + Send + Sync> std::fmt::Display for DrawingErrorKind<E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DrawingErrorKind::DrawingError(e) => write!(fmt, "Drawing backend error: {}", e),
            DrawingErrorKind::FontError(e) => write!(fmt, "Font loading error: {}", e),
        }
    }
}

impl<E: Error + Send + Sync> Error for DrawingErrorKind<E> {}

/// The style data for the backend drawing API
pub trait BackendStyle {
    /// The underlying type reprsents the color for this style
    type ColorType: Color;

    /// Convert the style into the underlying color
    fn as_color(&self) -> RGBAColor;
    // TODO: In the future we should support stroke width, line shape, etc....
}

impl<T: Color> BackendStyle for T {
    type ColorType = T;
    fn as_color(&self) -> RGBAColor {
        self.to_rgba()
    }
}

///  The drawing backend trait, which implemenets the low-level drawing APIs.
///  This trait has a set of default implementation. And the minimal requirement of
///  implementing a drawing backend is implementing the `draw_pixel` function.
///
///  If the drawing backend supports vector graphics, the other drawing APIs should be
///  overrided by the backend specific implementation. Otherwise, the default implementation
///  will use the pixel-based approach to draw other types of low-level shapes.
pub trait DrawingBackend {
    /// The error type reported by the backend
    type ErrorType: Error + Send + Sync;

    /// Get the dimension of the drawing backend in pixel
    fn get_size(&self) -> (u32, u32);

    /// Ensure the backend is ready to draw
    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Finialize the drawing step and present all the changes.
    /// This is used as the real-time rendering support.
    /// The backend may implement in the following way, when `ensure_prepared` is called
    /// it checks if it needs a fresh buffer and `present` is called rendering all the
    /// pending changes on the screen.
    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a pixel on the drawing backend
    /// - `point`: The backend pixel-based coordinate to draw
    /// - `color`: The color of the pixel
    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Draw a line on the drawing backend
    /// - `from`: The start point of the line
    /// - `to`: The end point of the line
    /// - `style`: The style of the line
    fn draw_line<S: BackendStyle>(
        &mut self,
        mut from: BackendCoord,
        mut to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }

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

        let grad = f64::from(to.1 - from.1) / f64::from(to.0 - from.0);

        let mut put_pixel = |(x, y): BackendCoord, b: f64| {
            if steep {
                self.draw_pixel((y, x), &style.as_color().mix(b))
            } else {
                self.draw_pixel((x, y), &style.as_color().mix(b))
            }
        };

        let mut y = f64::from(from.1);

        for x in from.0..=to.0 {
            put_pixel((x, y as i32), 1.0 + y.floor() - y)?;
            put_pixel((x, y as i32 + 1), y - y.floor())?;

            y += grad;
        }

        Ok(())
    }

    /// Draw a rectangle on the drawing backend
    /// - `upper_left`: The coordinate of the upper-left corner of the rect
    /// - `bottom_right`: The coordinate of the bottom-right corner of the rect
    /// - `style`: The style
    /// - `fill`: If the rectangle should be filled
    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }
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
                    self.draw_line((x, upper_left.1), (x, bottom_right.1), style)?;
                }
            } else {
                for y in upper_left.1..=bottom_right.1 {
                    self.draw_line((upper_left.0, y), (bottom_right.0, y), style)?;
                }
            }
        } else {
            self.draw_line(
                (upper_left.0, upper_left.1),
                (upper_left.0, bottom_right.1),
                style,
            )?;
            self.draw_line(
                (upper_left.0, upper_left.1),
                (bottom_right.0, upper_left.1),
                style,
            )?;
            self.draw_line(
                (bottom_right.0, bottom_right.1),
                (upper_left.0, bottom_right.1),
                style,
            )?;
            self.draw_line(
                (bottom_right.0, bottom_right.1),
                (bottom_right.0, upper_left.1),
                style,
            )?;
        }
        Ok(())
    }

    /// Draw a path on the drawing backend
    /// - `path`: The iterator of key points of the path
    /// - `style`: The style of the path
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }

        let mut begin: Option<BackendCoord> = None;
        for end in path.into_iter() {
            if let Some(begin) = begin {
                self.draw_line(begin, end, style)?;
            }
            begin = Some(end);
        }
        Ok(())
    }

    /// Draw a circle on the drawing backend
    /// - `center`: The center coordinate of the circle
    /// - `radius`: The radius of the circle
    /// - `style`: The style of the shape
    /// - `fill`: If the circle should be filled
    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.as_color().alpha() == 0.0 {
            return Ok(());
        }

        let min = (f64::from(radius) * (1.0 - (2f64).sqrt() / 2.0)).ceil() as i32;
        let max = (f64::from(radius) * (1.0 + (2f64).sqrt() / 2.0)).floor() as i32;

        let range = min..=max;

        let (up, down) = (
            range.start() + center.1 - radius as i32,
            range.end() + center.1 - radius as i32,
        );

        for dy in range {
            let dy = dy - radius as i32;
            let y = center.1 + dy;

            let lx = (f64::from(radius) * f64::from(radius)
                - (f64::from(dy) * f64::from(dy)).max(1e-5))
            .sqrt();

            let left = center.0 - lx.floor() as i32;
            let right = center.0 + lx.floor() as i32;

            let v = lx - lx.floor();

            let x = center.0 + dy;
            let top = center.1 - lx.floor() as i32;
            let bottom = center.1 + lx.floor() as i32;

            if fill {
                self.draw_line((left, y), (right, y), style)?;
                self.draw_line((x, top), (x, up), style)?;
                self.draw_line((x, down), (x, bottom), style)?;
            } else {
                self.draw_pixel((left, y), &style.as_color().mix(1.0 - v))?;
                self.draw_pixel((right, y), &style.as_color().mix(1.0 - v))?;

                self.draw_pixel((x, top), &style.as_color().mix(1.0 - v))?;
                self.draw_pixel((x, bottom), &style.as_color().mix(1.0 - v))?;
            }

            self.draw_pixel((left - 1, y), &style.as_color().mix(v))?;
            self.draw_pixel((right + 1, y), &style.as_color().mix(v))?;
            self.draw_pixel((x, top - 1), &style.as_color().mix(v))?;
            self.draw_pixel((x, bottom + 1), &style.as_color().mix(v))?;
        }

        Ok(())
    }

    /// Draw a text on the drawing backend
    /// - `text`: The text to draw
    /// - `font`: The description of the font
    /// - `pos` : The position backend
    /// - `color`: The color of the text
    fn draw_text<'a>(
        &mut self,
        text: &str,
        font: &FontDesc<'a>,
        pos: BackendCoord,
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if color.alpha() == 0.0 {
            return Ok(());
        }

        match font.draw(text, (pos.0, pos.1), |x, y, v| {
            self.draw_pixel((x as i32, y as i32), &color.mix(f64::from(v)))
        }) {
            Ok(drawing_result) => drawing_result,
            Err(font_error) => Err(DrawingErrorKind::FontError(font_error)),
        }
    }
}
