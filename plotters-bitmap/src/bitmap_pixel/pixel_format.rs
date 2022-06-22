use crate::BitMapBackend;
use plotters_backend::DrawingBackend;


#[inline(always)]
pub(super) fn blend(prev: &mut u8, new: u8, a: u64) {
    if new > *prev {
        *prev += (u64::from(new - *prev) * a / 256) as u8
    } else {
        *prev -= (u64::from(*prev - new) * a / 256) as u8
    }
}


/// The trait that describes some details about a particular pixel format
pub trait PixelFormat: Sized {
    /// Number of bytes per pixel
    const PIXEL_SIZE: usize;

    /// Number of effective bytes per pixel, e.g. for BGRX pixel format, the size of pixel
    /// is 4 but the effective size is 3, since the 4th byte isn't used
    const EFFECTIVE_PIXEL_SIZE: usize;

    /// Encoding a pixel and returns the idx-th byte for the pixel
    fn byte_at(r: u8, g: u8, b: u8, a: u64, idx: usize) -> u8;

    /// Decode a pixel at the given location
    fn decode_pixel(data: &[u8]) -> (u8, u8, u8, u64);

    /// The fast alpha blending algorithm for this pixel format
    ///
    /// - `target`: The target bitmap backend
    /// - `upper_left`: The upper-left coord for the rect
    /// - `bottom_right`: The bottom-right coord for the rect
    /// - `r`, `g`, `b`, `a`: The blending color and alpha value
    fn blend_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
        a: f64,
    );

    /// The fast vertical line filling algorithm
    ///
    /// - `target`: The target bitmap backend
    /// - `x`: the X coordinate for the entire line
    /// - `ys`: The range of y coord
    /// - `r`, `g`, `b`: The blending color and alpha value
    fn fill_vertical_line_fast(
        target: &mut BitMapBackend<'_, Self>,
        x: i32,
        ys: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
    ) {
        let (w, h) = target.get_size();
        let w = w as i32;
        let h = h as i32;

        // Make sure we are in the range
        if x < 0 || x >= w {
            return;
        }

        let dst = target.get_raw_pixel_buffer();
        let (mut y0, mut y1) = ys;
        if y0 > y1 {
            std::mem::swap(&mut y0, &mut y1);
        }
        // And check the y axis isn't out of bound
        y0 = y0.max(0);
        y1 = y1.min(h - 1);
        // This is ok because once y0 > y1, there won't be any iteration anymore
        for y in y0..=y1 {
            for idx in 0..Self::EFFECTIVE_PIXEL_SIZE {
                dst[(y * w + x) as usize * Self::PIXEL_SIZE + idx] = Self::byte_at(r, g, b, 0, idx);
            }
        }
    }

    /// The fast rectangle filling algorithm
    ///
    /// - `target`: The target bitmap backend
    /// - `upper_left`: The upper-left coord for the rect
    /// - `bottom_right`: The bottom-right coord for the rect
    /// - `r`, `g`, `b`: The filling color
    fn fill_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
    );

    #[inline(always)]
    /// Drawing a single pixel in this format
    ///
    /// - `target`: The target bitmap backend
    /// - `point`: The coord of the point
    /// - `r`, `g`, `b`: The filling color
    /// - `alpha`: The alpha value
    fn draw_pixel(
        target: &mut BitMapBackend<'_, Self>,
        point: (i32, i32),
        (r, g, b): (u8, u8, u8),
        alpha: f64,
    ) {
        let (x, y) = (point.0 as usize, point.1 as usize);
        let (w, _) = target.get_size();
        let buf = target.get_raw_pixel_buffer();
        let w = w as usize;
        let base = (y * w + x) * Self::PIXEL_SIZE;

        if base < buf.len() {
            unsafe {
                if alpha >= 1.0 - 1.0 / 256.0 {
                    for idx in 0..Self::EFFECTIVE_PIXEL_SIZE {
                        *buf.get_unchecked_mut(base + idx) = Self::byte_at(r, g, b, 0, idx);
                    }
                } else {
                    if alpha <= 0.0 {
                        return;
                    }

                    let alpha = (alpha * 256.0).floor() as u64;
                    for idx in 0..Self::EFFECTIVE_PIXEL_SIZE {
                        blend(
                            buf.get_unchecked_mut(base + idx),
                            Self::byte_at(r, g, b, 0, idx),
                            alpha,
                        );
                    }
                }
            }
        }
    }

    /// Indicates if this pixel format can be saved as image.
    /// Note: Currently we only using RGB pixel format in the image crate, but later we may lift
    /// this restriction
    ///
    /// - `returns`: If the image can be saved as image file
    fn can_be_saved() -> bool {
        false
    }
}

