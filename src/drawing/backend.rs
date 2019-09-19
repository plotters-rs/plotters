use crate::style::{Color, FontDesc, FontError, RGBAColor, ShapeStyle};
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
    fn stroke_width(&self) -> u32 { 1 }
}

impl<T: Color> BackendStyle for T {
    type ColorType = T;
    fn as_color(&self) -> RGBAColor {
        self.to_rgba()
    }
}

impl BackendStyle for ShapeStyle {
    type ColorType = RGBAColor;
    fn as_color(&self) -> RGBAColor {
        self.color.clone()
    }
    fn stroke_width(&self) -> u32 {
        self.stroke_width
    }
}

///  The drawing backend trait, which implemenets the low-level drawing APIs.
///  This trait has a set of default implementation. And the minimal requirement of
///  implementing a drawing backend is implementing the `draw_pixel` function.
///
///  If the drawing backend supports vector graphics, the other drawing APIs should be
///  overrided by the backend specific implementation. Otherwise, the default implementation
///  will use the pixel-based approach to draw other types of low-level shapes.
pub trait DrawingBackend: Sized {
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
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        super::rasterizer::draw_line(self, from, to, style)
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
        super::rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
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

        if style.stroke_width() == 1 {
            let mut begin: Option<BackendCoord> = None;
            for end in path.into_iter() {
                if let Some(begin) = begin {
                    self.draw_line(begin, end, style)?;
                }
                begin = Some(end);
            }
        } else {
            let p:Vec<_> = path.into_iter().collect();
            let v = super::rasterizer::path::polygonize(&p[..], style.stroke_width());
            return self.fill_polygon(v, &style.as_color());
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
        super::rasterizer::draw_circle(self, center, radius, style, fill)
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let vert_buf: Vec<_> = vert.into_iter().collect();

        super::rasterizer::fill_polygon(self, &vert_buf[..], style)
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
