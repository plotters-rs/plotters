use crate::style::text_anchor::{HPos, VPos};
use std::error::Error;
//use crate::style::{FontDesc, FontError, TextStyle};

pub mod rasterizer;
mod style;
mod text;

pub use style::{BackendColor, BackendStyle};
pub use text::{text_anchor, BackendTextStyle, FontFamily, FontStyle, FontTransform};

/// A coordinate in the image
pub type BackendCoord = (i32, i32);

/// The error produced by a drawing backend
#[derive(Debug)]
pub enum DrawingErrorKind<E: Error + Send + Sync> {
    /// A drawing backend error
    DrawingError(E),
    /// A font rendering error
    FontError(Box<dyn Error + Send + Sync + 'static>),
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

///  The drawing backend trait, which implements the low-level drawing APIs.
///  This trait has a set of default implementation. And the minimal requirement of
///  implementing a drawing backend is implementing the `draw_pixel` function.
///
///  If the drawing backend supports vector graphics, the other drawing APIs should be
///  override by the backend specific implementation. Otherwise, the default implementation
///  will use the pixel-based approach to draw other types of low-level shapes.
pub trait DrawingBackend: Sized {
    /// The error type reported by the backend
    type ErrorType: Error + Send + Sync;

    /// Get the dimension of the drawing backend in pixels
    fn get_size(&self) -> (u32, u32);

    /// Ensure the backend is ready to draw
    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>>;

    /// Finalize the drawing step and present all the changes.
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
        color: BackendColor,
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
        rasterizer::draw_line(self, from, to, style)
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
        rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
    }

    /// Draw a path on the drawing backend
    /// - `path`: The iterator of key points of the path
    /// - `style`: The style of the path
    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }

        if style.stroke_width() == 1 {
            let mut begin: Option<BackendCoord> = None;
            for end in path.into_iter() {
                if let Some(begin) = begin {
                    let result = self.draw_line(begin, end, style);
                    if result.is_err() {
                        return result;
                    }
                }
                begin = Some(end);
            }
        } else {
            let p: Vec<_> = path.into_iter().collect();
            let v = rasterizer::polygonize(&p[..], style.stroke_width());
            return self.fill_polygon(v, &style.color());
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
        rasterizer::draw_circle(self, center, radius, style, fill)
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let vert_buf: Vec<_> = vert.into_iter().collect();

        rasterizer::fill_polygon(self, &vert_buf[..], style)
    }

    /// Draw a text on the drawing backend
    /// - `text`: The text to draw
    /// - `style`: The text style
    /// - `pos` : The text anchor point
    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }

        let layout = style
            .layout_box(text)
            .map_err(|e| DrawingErrorKind::FontError(Box::new(e)))?;
        let ((min_x, min_y), (max_x, max_y)) = layout;
        let width = (max_x - min_x) as i32;
        let height = (max_y - min_y) as i32;
        let dx = match style.anchor().h_pos {
            HPos::Left => 0,
            HPos::Right => -width,
            HPos::Center => -width / 2,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0,
            VPos::Center => -height / 2,
            VPos::Bottom => -height,
        };
        let trans = style.transform();
        let (w, h) = self.get_size();
        match style.draw(text, (0, 0), |x, y, color| {
            let (x, y) = trans.transform(x + dx - min_x, y + dy - min_y);
            let (x, y) = (pos.0 + x, pos.1 + y);
            if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                self.draw_pixel((x, y), color)
            } else {
                Ok(())
            }
        }) {
            Ok(drawing_result) => drawing_result,
            Err(font_error) => Err(DrawingErrorKind::FontError(Box::new(font_error))),
        }
    }

    /// Estimate the size of the horizontal text if rendered on this backend.
    /// This is important because some of the backend may not have font ability.
    /// Thus this allows those backend reports proper value rather than ask the
    /// font rasterizer for that.
    ///
    /// - `text`: The text to estimate
    /// - `font`: The font to estimate
    /// - *Returns* The estimated text size
    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let layout = style
            .layout_box(text)
            .map_err(|e| DrawingErrorKind::FontError(Box::new(e)))?;
        Ok((
            ((layout.1).0 - (layout.0).0) as u32,
            ((layout.1).1 - (layout.0).1) as u32,
        ))
    }

    /// Blit a bitmap on to the backend.
    ///
    /// - `text`: pos the left upper conner of the bitmap to blit
    /// - `src`: The source of the image
    ///
    /// TODO: The default implementation of bitmap blitting assumes that the bitmap is RGB, but
    /// this may not be the case. But for bitmap backend it's actually ok if we use the bitmap
    /// element that matches the pixel format, but we need to fix this.
    fn blit_bitmap<'a>(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &'a [u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (w, h) = self.get_size();

        for dx in 0..iw {
            if pos.0 + dx as i32 >= w as i32 {
                break;
            }
            for dy in 0..ih {
                if pos.1 + dy as i32 >= h as i32 {
                    break;
                }
                // FIXME: This assume we have RGB image buffer
                let r = src[(dx + dy * w) as usize * 3];
                let g = src[(dx + dy * w) as usize * 3 + 1];
                let b = src[(dx + dy * w) as usize * 3 + 2];
                let color = BackendColor {
                    alpha: 1.0,
                    rgb: (r, g, b),
                };
                let result = self.draw_pixel((pos.0 + dx as i32, pos.1 + dy as i32), color);
                if result.is_err() {
                    return result;
                }
            }
        }

        Ok(())
    }
}
