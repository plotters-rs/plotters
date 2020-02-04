use crate::plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind,
};
use std::marker::PhantomData;

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
mod image_encoding_support {
    pub(super) use image::{ImageBuffer, ImageError, Rgb};
    pub(super) use std::path::Path;
    pub(super) type BorrowedImage<'a> = ImageBuffer<Rgb<u8>, &'a mut [u8]>;
}

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
use image_encoding_support::*;

#[derive(Debug)]
/// Indicates some error occurs within the bitmap backend
pub enum BitMapBackendError {
    /// The buffer provided is invalid, for example, wrong pixel buffer size
    InvalidBuffer,
    /// Some IO error occurs while the bitmap maniuplation
    IOError(std::io::Error),
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    /// Image encoding error
    ImageError(ImageError),
}

impl std::fmt::Display for BitMapBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BitMapBackendError {}

#[inline(always)]
fn blend(prev: &mut u8, new: u8, a: u64) {
    if new > *prev {
        *prev += (u64::from(new - *prev) * a / 256) as u8
    } else {
        *prev -= (u64::from(*prev - new) * a / 256) as u8
    }
}

#[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
mod gif_support {
    use super::*;
    use gif::{Encoder as GifEncoder, Frame as GifFrame, Repeat, SetParameter};
    use std::fs::File;

    pub(super) struct GifFile {
        encoder: GifEncoder<File>,
        height: u32,
        width: u32,
        delay: u32,
    }

    impl GifFile {
        pub(super) fn new<T: AsRef<Path>>(
            path: T,
            dim: (u32, u32),
            delay: u32,
        ) -> Result<Self, BitMapBackendError> {
            let mut encoder = GifEncoder::new(
                File::create(path.as_ref()).map_err(BitMapBackendError::IOError)?,
                dim.0 as u16,
                dim.1 as u16,
                &[],
            )
            .map_err(BitMapBackendError::IOError)?;

            encoder
                .set(Repeat::Infinite)
                .map_err(BitMapBackendError::IOError)?;

            Ok(Self {
                encoder,
                width: dim.0,
                height: dim.1,
                delay: (delay + 5) / 10,
            })
        }

        pub(super) fn flush_frame(&mut self, buffer: &[u8]) -> Result<(), BitMapBackendError> {
            let mut frame =
                GifFrame::from_rgb_speed(self.width as u16, self.height as u16, buffer, 10);

            frame.delay = self.delay as u16;

            self.encoder
                .write_frame(&frame)
                .map_err(BitMapBackendError::IOError)?;

            Ok(())
        }
    }
}

enum Target<'a> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    File(&'a Path),
    Buffer(PhantomData<&'a u32>),
    #[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
    Gif(Box<gif_support::GifFile>),
}

enum Buffer<'a> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    Owned(Vec<u8>),
    Borrowed(&'a mut [u8]),
}

impl<'a> Buffer<'a> {
    #[inline(always)]
    fn borrow_buffer(&mut self) -> &mut [u8] {
        match self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
            Buffer::Owned(buf) => &mut buf[..],
            Buffer::Borrowed(buf) => *buf,
        }
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

/// The marker type that indicates we are currently using a RGB888 pixel format
pub struct RGBPixel;

/// The marker type that indicates we are currently using a BGRX8888 pixel format
pub struct BGRXPixel;

impl PixelFormat for RGBPixel {
    const PIXEL_SIZE: usize = 3;
    const EFFECTIVE_PIXEL_SIZE: usize = 3;

    #[inline(always)]
    fn byte_at(r: u8, g: u8, b: u8, _a: u64, idx: usize) -> u8 {
        match idx {
            0 => r,
            1 => g,
            2 => b,
            _ => 0xff,
        }
    }

    #[inline(always)]
    fn decode_pixel(data: &[u8]) -> (u8, u8, u8, u64) {
        (data[0], data[1], data[2], 0x255)
    }

    fn can_be_saved() -> bool {
        true
    }

    #[allow(clippy::many_single_char_names, clippy::cast_ptr_alignment)]
    fn blend_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
        a: f64,
    ) {
        let (w, h) = target.get_size();
        let a = a.min(1.0).max(0.0);
        if a == 0.0 {
            return;
        }

        let (x0, y0) = (
            upper_left.0.min(bottom_right.0).max(0),
            upper_left.1.min(bottom_right.1).max(0),
        );
        let (x1, y1) = (
            upper_left.0.max(bottom_right.0).min(w as i32 - 1),
            upper_left.1.max(bottom_right.1).min(h as i32 - 1),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 > x1 || y0 > y1 {
            return;
        }

        let dst = target.get_raw_pixel_buffer();

        let a = (256.0 * a).floor() as u64;

        // Since we should always make sure the RGB payload occupies the logic lower bits
        // thus, this type purning should work for both LE and BE CPUs
        #[rustfmt::skip]
        let (p1, p2, p3): (u64, u64, u64) = unsafe {
            std::mem::transmute([
                u16::from(r), u16::from(b), u16::from(g), u16::from(r), // QW1
                u16::from(b), u16::from(g), u16::from(r), u16::from(b), // QW2
                u16::from(g), u16::from(r), u16::from(b), u16::from(g), // QW3
            ])
        };

        #[rustfmt::skip]
        let (q1, q2, q3): (u64, u64, u64) = unsafe {
            std::mem::transmute([
                u16::from(g), u16::from(r), u16::from(b), u16::from(g), // QW1
                u16::from(r), u16::from(b), u16::from(g), u16::from(r), // QW2
                u16::from(b), u16::from(g), u16::from(r), u16::from(b), // QW3
            ])
        };

        const N: u64 = 0xff00_ff00_ff00_ff00;
        const M: u64 = 0x00ff_00ff_00ff_00ff;

        for y in y0..=y1 {
            let start = (y * w as i32 + x0) as usize;
            let count = (x1 - x0 + 1) as usize;

            let start_ptr = &mut dst[start * Self::PIXEL_SIZE] as *mut u8 as *mut [u8; 24];
            let slice = unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 8) };
            for p in slice.iter_mut() {
                let ptr = p as *mut [u8; 24] as *mut (u64, u64, u64);
                let (d1, d2, d3) = unsafe { *ptr };
                let (mut h1, mut h2, mut h3) = ((d1 >> 8) & M, (d2 >> 8) & M, (d3 >> 8) & M);
                let (mut l1, mut l2, mut l3) = (d1 & M, d2 & M, d3 & M);

                #[cfg(target_endian = "little")]
                {
                    h1 = (h1 * (256 - a) + q1 * a) & N;
                    h2 = (h2 * (256 - a) + q2 * a) & N;
                    h3 = (h3 * (256 - a) + q3 * a) & N;
                    l1 = ((l1 * (256 - a) + p1 * a) & N) >> 8;
                    l2 = ((l2 * (256 - a) + p2 * a) & N) >> 8;
                    l3 = ((l3 * (256 - a) + p3 * a) & N) >> 8;
                }

                #[cfg(target_endian = "big")]
                {
                    h1 = (h1 * (256 - a) + p1 * a) & N;
                    h2 = (h2 * (256 - a) + p2 * a) & N;
                    h3 = (h3 * (256 - a) + p3 * a) & N;
                    l1 = ((l1 * (256 - a) + q1 * a) & N) >> 8;
                    l2 = ((l2 * (256 - a) + q2 * a) & N) >> 8;
                    l3 = ((l3 * (256 - a) + q3 * a) & N) >> 8;
                }

                unsafe {
                    *ptr = (h1 | l1, h2 | l2, h3 | l3);
                }
            }

            let mut iter = dst[((start + slice.len() * 8) * Self::PIXEL_SIZE)
                ..((start + count) * Self::PIXEL_SIZE)]
                .iter_mut();
            for _ in (slice.len() * 8)..count {
                blend(iter.next().unwrap(), r, a);
                blend(iter.next().unwrap(), g, a);
                blend(iter.next().unwrap(), b, a);
            }
        }
    }

    #[allow(clippy::many_single_char_names, clippy::cast_ptr_alignment)]
    fn fill_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
    ) {
        let (w, h) = target.get_size();
        let (x0, y0) = (
            upper_left.0.min(bottom_right.0).max(0),
            upper_left.1.min(bottom_right.1).max(0),
        );
        let (x1, y1) = (
            upper_left.0.max(bottom_right.0).min(w as i32 - 1),
            upper_left.1.max(bottom_right.1).min(h as i32 - 1),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 > x1 || y0 > y1 {
            return;
        }

        let dst = target.get_raw_pixel_buffer();

        if r == g && g == b {
            // If r == g == b, then we can use memset
            if x0 != 0 || x1 != w as i32 - 1 {
                // If it's not the entire row is filled, we can only do
                // memset per row
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let count = (x1 - x0 + 1) as usize;
                    dst[(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut()
                        .for_each(|e| *e = r);
                }
            } else {
                // If the entire memory block is going to be filled, just use single memset
                dst[Self::PIXEL_SIZE * (y0 * w as i32) as usize
                    ..((y1 + 1) * w as i32) as usize * Self::PIXEL_SIZE]
                    .iter_mut()
                    .for_each(|e| *e = r);
            }
        } else {
            let count = (x1 - x0 + 1) as usize;
            if count < 8 {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let mut iter = dst
                        [(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut();
                    for _ in 0..=(x1 - x0) {
                        *iter.next().unwrap() = r;
                        *iter.next().unwrap() = g;
                        *iter.next().unwrap() = b;
                    }
                }
            } else {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let start_ptr = &mut dst[start * Self::PIXEL_SIZE] as *mut u8 as *mut [u8; 24];
                    let slice =
                        unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 8) };
                    for p in slice.iter_mut() {
                        // In this case, we can actually fill 8 pixels in one iteration with
                        // only 3 movq instructions.
                        // TODO: Consider using AVX instructions when possible
                        let ptr = p as *mut [u8; 24] as *mut u64;
                        unsafe {
                            let (d1, d2, d3): (u64, u64, u64) = std::mem::transmute([
                                r, g, b, r, g, b, r, g, // QW1
                                b, r, g, b, r, g, b, r, // QW2
                                g, b, r, g, b, r, g, b, // QW3
                            ]);
                            *ptr = d1;
                            *ptr.offset(1) = d2;
                            *ptr.offset(2) = d3;
                        }
                    }

                    for idx in (slice.len() * 8)..count {
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE] = r;
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE + 1] = g;
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE + 2] = b;
                    }
                }
            }
        }
    }
}

impl PixelFormat for BGRXPixel {
    const PIXEL_SIZE: usize = 4;
    const EFFECTIVE_PIXEL_SIZE: usize = 3;

    #[inline(always)]
    fn byte_at(r: u8, g: u8, b: u8, _a: u64, idx: usize) -> u8 {
        match idx {
            0 => b,
            1 => g,
            2 => r,
            _ => 0xff,
        }
    }

    #[inline(always)]
    fn decode_pixel(data: &[u8]) -> (u8, u8, u8, u64) {
        (data[2], data[1], data[0], 0x255)
    }

    #[allow(clippy::many_single_char_names, clippy::cast_ptr_alignment)]
    fn blend_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
        a: f64,
    ) {
        let (w, h) = target.get_size();
        let a = a.min(1.0).max(0.0);
        if a == 0.0 {
            return;
        }

        let (x0, y0) = (
            upper_left.0.min(bottom_right.0).max(0),
            upper_left.1.min(bottom_right.1).max(0),
        );
        let (x1, y1) = (
            upper_left.0.max(bottom_right.0).min(w as i32 - 1),
            upper_left.1.max(bottom_right.1).min(h as i32 - 1),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 > x1 || y0 > y1 {
            return;
        }

        let dst = target.get_raw_pixel_buffer();

        let a = (256.0 * a).floor() as u64;

        // Since we should always make sure the RGB payload occupies the logic lower bits
        // thus, this type purning should work for both LE and BE CPUs
        #[rustfmt::skip]
        let p: u64 = unsafe {
            std::mem::transmute([
                u16::from(b), u16::from(r), u16::from(b), u16::from(r), // QW1
            ])
        };

        #[rustfmt::skip]
        let q: u64 = unsafe {
            std::mem::transmute([
                u16::from(g), 0u16, u16::from(g), 0u16, // QW1
            ])
        };

        const N: u64 = 0xff00_ff00_ff00_ff00;
        const M: u64 = 0x00ff_00ff_00ff_00ff;

        for y in y0..=y1 {
            let start = (y * w as i32 + x0) as usize;
            let count = (x1 - x0 + 1) as usize;

            let start_ptr = &mut dst[start * Self::PIXEL_SIZE] as *mut u8 as *mut [u8; 8];
            let slice = unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 2) };
            for rp in slice.iter_mut() {
                let ptr = rp as *mut [u8; 8] as *mut u64;
                let d1 = unsafe { *ptr };
                let mut h = (d1 >> 8) & M;
                let mut l = d1 & M;

                #[cfg(target_endian = "little")]
                {
                    h = (h * (256 - a) + q * a) & N;
                    l = ((l * (256 - a) + p * a) & N) >> 8;
                }

                #[cfg(target_endian = "big")]
                {
                    h = (h * (256 - a) + p * a) & N;
                    l = ((l * (256 - a) + q * a) & N) >> 8;
                }

                unsafe {
                    *ptr = h | l;
                }
            }

            let mut iter = dst[((start + slice.len() * 2) * Self::PIXEL_SIZE)
                ..((start + count) * Self::PIXEL_SIZE)]
                .iter_mut();
            for _ in (slice.len() * 2)..count {
                blend(iter.next().unwrap(), b, a);
                blend(iter.next().unwrap(), g, a);
                blend(iter.next().unwrap(), r, a);
                iter.next();
            }
        }
    }

    #[allow(clippy::many_single_char_names, clippy::cast_ptr_alignment)]
    fn fill_rect_fast(
        target: &mut BitMapBackend<'_, Self>,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
    ) {
        let (w, h) = target.get_size();
        let (x0, y0) = (
            upper_left.0.min(bottom_right.0).max(0),
            upper_left.1.min(bottom_right.1).max(0),
        );
        let (x1, y1) = (
            upper_left.0.max(bottom_right.0).min(w as i32 - 1),
            upper_left.1.max(bottom_right.1).min(h as i32 - 1),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 > x1 || y0 > y1 {
            return;
        }

        let dst = target.get_raw_pixel_buffer();

        if r == g && g == b {
            // If r == g == b, then we can use memset
            if x0 != 0 || x1 != w as i32 - 1 {
                // If it's not the entire row is filled, we can only do
                // memset per row
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let count = (x1 - x0 + 1) as usize;
                    dst[(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut()
                        .for_each(|e| *e = r);
                }
            } else {
                // If the entire memory block is going to be filled, just use single memset
                dst[Self::PIXEL_SIZE * (y0 * w as i32) as usize
                    ..((y1 + 1) * w as i32) as usize * Self::PIXEL_SIZE]
                    .iter_mut()
                    .for_each(|e| *e = r);
            }
        } else {
            let count = (x1 - x0 + 1) as usize;
            if count < 8 {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let mut iter = dst
                        [(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut();
                    for _ in 0..=(x1 - x0) {
                        *iter.next().unwrap() = b;
                        *iter.next().unwrap() = g;
                        *iter.next().unwrap() = r;
                        iter.next();
                    }
                }
            } else {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let start_ptr = &mut dst[start * Self::PIXEL_SIZE] as *mut u8 as *mut [u8; 8];
                    let slice =
                        unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 2) };
                    for p in slice.iter_mut() {
                        // In this case, we can actually fill 8 pixels in one iteration with
                        // only 3 movq instructions.
                        // TODO: Consider using AVX instructions when possible
                        let ptr = p as *mut [u8; 8] as *mut u64;
                        unsafe {
                            let d: u64 = std::mem::transmute([
                                b, g, r, 0, b, g, r, 0, // QW1
                            ]);
                            *ptr = d;
                        }
                    }

                    for idx in (slice.len() * 2)..count {
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE] = b;
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE + 1] = g;
                        dst[start * Self::PIXEL_SIZE + idx * Self::PIXEL_SIZE + 2] = r;
                    }
                }
            }
        }
    }
}

/// The backend that drawing a bitmap
pub struct BitMapBackend<'a, P: PixelFormat = RGBPixel> {
    /// The path to the image
    #[allow(dead_code)]
    target: Target<'a>,
    /// The size of the image
    size: (u32, u32),
    /// The data buffer of the image
    buffer: Buffer<'a>,
    /// Flag indicates if the bitmap has been saved
    saved: bool,
    _pantomdata: PhantomData<P>,
}

impl<'a, P: PixelFormat> BitMapBackend<'a, P> {
    /// The number of bytes per pixel
    const PIXEL_SIZE: usize = P::PIXEL_SIZE;
}

impl<'a> BitMapBackend<'a, RGBPixel> {
    /// Create a new bitmap backend
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, (w, h): (u32, u32)) -> Self {
        Self {
            target: Target::File(path.as_ref()),
            size: (w, h),
            buffer: Buffer::Owned(vec![0; Self::PIXEL_SIZE * (w * h) as usize]),
            saved: false,
            _pantomdata: PhantomData,
        }
    }

    /// Create a new bitmap backend that generate GIF animation
    ///
    /// When this is used, the bitmap backend acts similar to a real-time rendering backend.
    /// When the program finished drawing one frame, use `present` function to flush the frame
    /// into the GIF file.
    ///
    /// - `path`: The path to the GIF file to create
    /// - `dimension`: The size of the GIF image
    /// - `speed`: The amount of time for each frame to display
    #[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
    pub fn gif<T: AsRef<Path>>(
        path: T,
        (w, h): (u32, u32),
        frame_delay: u32,
    ) -> Result<Self, BitMapBackendError> {
        Ok(Self {
            target: Target::Gif(Box::new(gif_support::GifFile::new(
                path,
                (w, h),
                frame_delay,
            )?)),
            size: (w, h),
            buffer: Buffer::Owned(vec![0; Self::PIXEL_SIZE * (w * h) as usize]),
            saved: false,
            _pantomdata: PhantomData,
        })
    }

    /// Create a new bitmap backend which only lives in-memory
    ///
    /// When this is used, the bitmap backend will write to a user provided [u8] array (or Vec<u8>)
    /// in RGB pixel format.
    ///
    /// Note: This function provides backward compatibility for those code that assumes Plotters
    /// uses RGB pixel format and maniuplates the in-memory framebuffer.
    /// For more pixel format option, use `with_buffer_and_format` instead.
    ///
    /// - `buf`: The buffer to operate
    /// - `dimension`: The size of the image in pixels
    /// - **returns**: The newly created bitmap backend
    pub fn with_buffer(buf: &'a mut [u8], (w, h): (u32, u32)) -> Self {
        Self::with_buffer_and_format(buf, (w, h)).expect("Wrong buffer size")
    }
}

impl<'a, P: PixelFormat> BitMapBackend<'a, P> {
    /// Create a new bitmap backend with a in-memory buffer with specific pixel format.
    ///
    /// Note: This can be used as a way to manipulate framebuffer, `mmap` can be used on the top of this
    /// as well.
    ///
    /// - `buf`: The buffer to operate
    /// - `dimension`: The size of the image in pixels
    /// - **returns**: The newly created bitmap backend
    pub fn with_buffer_and_format(
        buf: &'a mut [u8],
        (w, h): (u32, u32),
    ) -> Result<Self, BitMapBackendError> {
        if (w * h) as usize * Self::PIXEL_SIZE > buf.len() {
            return Err(BitMapBackendError::InvalidBuffer);
        }

        Ok(Self {
            target: Target::Buffer(PhantomData),
            size: (w, h),
            buffer: Buffer::Borrowed(buf),
            saved: false,
            _pantomdata: PhantomData,
        })
    }

    #[inline(always)]
    fn get_raw_pixel_buffer(&mut self) -> &mut [u8] {
        self.buffer.borrow_buffer()
    }

    /// Split a bitmap backend vertically into several sub drawing area which allows
    /// multi-threading rendering.
    ///
    /// - `area_size`: The size of the area
    /// - **returns**: The splitted backends that can be rendered in parallel
    pub fn split(&mut self, area_size: &[u32]) -> Vec<BitMapBackend<P>> {
        let (w, h) = self.get_size();
        let buf = self.get_raw_pixel_buffer();

        let base_addr = &mut buf[0] as *mut u8;
        let mut split_points = vec![0];
        for size in area_size {
            let next = split_points.last().unwrap() + size;
            if next >= h {
                break;
            }
            split_points.push(next);
        }
        split_points.push(h);

        split_points
            .iter()
            .zip(split_points.iter().skip(1))
            .map(|(begin, end)| {
                let actual_buf = unsafe {
                    std::slice::from_raw_parts_mut(
                        base_addr.offset((begin * w) as isize * Self::PIXEL_SIZE as isize),
                        ((end - begin) * w) as usize * Self::PIXEL_SIZE,
                    )
                };
                Self::with_buffer_and_format(actual_buf, (w, end - begin)).unwrap()
            })
            .collect()
    }
}

impl<'a, P: PixelFormat> DrawingBackend for BitMapBackend<'a, P> {
    type ErrorType = BitMapBackendError;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<BitMapBackendError>> {
        self.saved = false;
        Ok(())
    }

    #[cfg(any(target_arch = "wasm32", not(feature = "image")))]
    fn present(&mut self) -> Result<(), DrawingErrorKind<BitMapBackendError>> {
        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    fn present(&mut self) -> Result<(), DrawingErrorKind<BitMapBackendError>> {
        if !P::can_be_saved() {
            return Ok(());
        }
        let (w, h) = self.get_size();
        match &mut self.target {
            Target::File(path) => {
                if let Some(img) = BorrowedImage::from_raw(w, h, self.buffer.borrow_buffer()) {
                    img.save(&path).map_err(|x| {
                        DrawingErrorKind::DrawingError(BitMapBackendError::IOError(x))
                    })?;
                    self.saved = true;
                    Ok(())
                } else {
                    Err(DrawingErrorKind::DrawingError(
                        BitMapBackendError::InvalidBuffer,
                    ))
                }
            }
            Target::Buffer(_) => Ok(()),

            #[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
            Target::Gif(target) => {
                target
                    .flush_frame(self.buffer.borrow_buffer())
                    .map_err(DrawingErrorKind::DrawingError)?;
                self.saved = true;
                Ok(())
            }
        }
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<BitMapBackendError>> {
        if point.0 < 0 || point.1 < 0 {
            return Ok(());
        }

        let alpha = color.alpha;
        let rgb = color.rgb;

        P::draw_pixel(self, point, rgb, alpha);

        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let alpha = style.color().alpha;
        let (r, g, b) = style.color().rgb;

        if (from.0 == to.0 || from.1 == to.1) && style.stroke_width() == 1 {
            if alpha >= 1.0 {
                if from.1 == to.1 {
                    P::fill_rect_fast(self, from, to, r, g, b);
                } else {
                    P::fill_vertical_line_fast(self, from.0, (from.1, to.1), r, g, b);
                }
            } else {
                P::blend_rect_fast(self, from, to, r, g, b, alpha);
            }
            return Ok(());
        }

        crate::plotters_backend::rasterizer::draw_line(self, from, to, style)
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let alpha = style.color().alpha;
        let (r, g, b) = style.color().rgb;
        if fill {
            if alpha >= 1.0 {
                P::fill_rect_fast(self, upper_left, bottom_right, r, g, b);
            } else {
                P::blend_rect_fast(self, upper_left, bottom_right, r, g, b, alpha);
            }
            return Ok(());
        }
        crate::plotters_backend::rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
    }

    fn blit_bitmap<'b>(
        &mut self,
        pos: BackendCoord,
        (sw, sh): (u32, u32),
        src: &'b [u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (dw, dh) = self.get_size();

        let (x0, y0) = pos;
        let (x1, y1) = (x0 + sw as i32, y0 + sh as i32);

        let (x0, y0, x1, y1) = (x0.max(0), y0.max(0), x1.min(dw as i32), y1.min(dh as i32));

        if x0 == x1 || y0 == y1 {
            return Ok(());
        }

        let mut chunk_size = (x1 - x0) as usize;
        let mut num_chunks = (y1 - y0) as usize;
        let dst_gap = dw as usize - chunk_size;
        let src_gap = sw as usize - chunk_size;

        let dst_start = Self::PIXEL_SIZE * (y0 as usize * dw as usize + x0 as usize);

        let mut dst = &mut self.get_raw_pixel_buffer()[dst_start..];

        let src_start =
            Self::PIXEL_SIZE * ((sh as i32 + y0 - y1) * sw as i32 + (sw as i32 + x0 - x1)) as usize;
        let mut src = &src[src_start..];

        if src_gap == 0 && dst_gap == 0 {
            chunk_size *= num_chunks;
            num_chunks = 1;
        }
        for i in 0..num_chunks {
            dst[0..(chunk_size * Self::PIXEL_SIZE)]
                .copy_from_slice(&src[0..(chunk_size * Self::PIXEL_SIZE)]);
            if i != num_chunks - 1 {
                dst = &mut dst[((chunk_size + dst_gap) * Self::PIXEL_SIZE)..];
                src = &src[((chunk_size + src_gap) * Self::PIXEL_SIZE)..];
            }
        }

        Ok(())
    }
}

impl<P: PixelFormat> Drop for BitMapBackend<'_, P> {
    fn drop(&mut self) {
        if !self.saved {
            self.present().expect("Unable to save the bitmap");
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_backend() {
    use crate::prelude::*;
    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.fill(&WHITE).unwrap();
        area.draw(&PathElement::new(vec![(0, 0), (10, 10)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }

    for i in 0..10 {
        assert_eq!(buffer[i * 33], 255);
        assert_eq!(buffer[i * 33 + 1], 0);
        assert_eq!(buffer[i * 33 + 2], 0);
        buffer[i * 33] = 255;
        buffer[i * 33 + 1] = 255;
        buffer[i * 33 + 2] = 255;
    }

    assert!(buffer.into_iter().all(|x| x == 255));
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_fill_half() {
    use crate::prelude::*;
    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new([(0, 0), (5, 10)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }
    for x in 0..10 {
        for y in 0..10 {
            assert_eq!(
                buffer[(y * 10 + x) as usize * 3 + 0],
                if x <= 5 { 255 } else { 0 }
            );
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], 0);
        }
    }

    let mut buffer = vec![0; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new([(0, 0), (10, 5)], RED.filled()))
            .unwrap();
        area.present().unwrap();
    }
    for x in 0..10 {
        for y in 0..10 {
            assert_eq!(
                buffer[(y * 10 + x) as usize * 3 + 0],
                if y <= 5 { 255 } else { 0 }
            );
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], 0);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_blend() {
    use crate::prelude::*;
    let mut buffer = vec![255; 10 * 10 * 3];

    {
        let back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        let area = back.into_drawing_area();
        area.draw(&Rectangle::new(
            [(0, 0), (5, 10)],
            RGBColor(0, 100, 200).mix(0.2).filled(),
        ))
        .unwrap();
        area.present().unwrap();
    }

    for x in 0..10 {
        for y in 0..10 {
            let (r, g, b) = if x <= 5 {
                (205, 225, 245)
            } else {
                (255, 255, 255)
            };
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 0], r);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], g);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], b);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_backend_split_and_fill() {
    use crate::prelude::*;
    let mut buffer = vec![255; 10 * 10 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (10, 10));

        for (sub_backend, color) in back.split(&[5]).into_iter().zip([&RED, &GREEN].iter()) {
            sub_backend.into_drawing_area().fill(*color).unwrap();
        }
    }

    for x in 0..10 {
        for y in 0..10 {
            let (r, g, b) = if y < 5 { (255, 0, 0) } else { (0, 255, 0) };
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 0], r);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 1], g);
            assert_eq!(buffer[(y * 10 + x) as usize * 3 + 2], b);
        }
    }
}

#[cfg(test)]
#[test]
fn test_draw_rect_out_of_range() {
    use crate::prelude::*;
    let mut buffer = vec![0; 1099 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

        back.draw_line((1100, 0), (1100, 999), &RED.to_rgba())
            .unwrap();
        back.draw_line((0, 1100), (999, 1100), &RED.to_rgba())
            .unwrap();
        back.draw_rect((1100, 0), (1100, 999), &RED.to_rgba(), true)
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], 0);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], 0);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], 0);
        }
    }
}

#[cfg(test)]
#[test]
fn test_draw_line_out_of_range() {
    use crate::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

        back.draw_line((-1000, -1000), (2000, 2000), &WHITE.to_rgba())
            .unwrap();

        back.draw_line((999, -1000), (999, 2000), &WHITE.to_rgba())
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            let expected_value = if x == y || x == 999 { 255 } else { 0 };
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], expected_value);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], expected_value);
            assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], expected_value);
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_blend_large() {
    use crate::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    for fill_color in [RED, GREEN, BLUE].iter() {
        buffer.iter_mut().for_each(|x| *x = 0);

        {
            let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));

            back.draw_rect((0, 0), (1000, 1000), &WHITE.mix(0.1), true)
                .unwrap(); // should be (24, 24, 24)
            back.draw_rect((0, 0), (100, 100), &fill_color.mix(0.5), true)
                .unwrap(); // should be (139, 24, 24)
        }

        for x in 0..1000 {
            for y in 0..1000 {
                let expected_value = if x <= 100 && y <= 100 {
                    let (r, g, b) = fill_color.to_rgba().rgb();
                    (
                        if r > 0 { 139 } else { 12 },
                        if g > 0 { 139 } else { 12 },
                        if b > 0 { 139 } else { 12 },
                    )
                } else {
                    (24, 24, 24)
                };
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 0], expected_value.0);
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 1], expected_value.1);
                assert_eq!(buffer[(y * 1000 + x) as usize * 3 + 2], expected_value.2);
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_bitmap_bgrx_pixel_format() {
    use crate::drawing::bitmap_pixel::BGRXPixel;
    use crate::prelude::*;
    let mut rgb_buffer = vec![0; 1000 * 1000 * 3];
    let mut bgrx_buffer = vec![0; 1000 * 1000 * 4];

    {
        let mut rgb_back = BitMapBackend::with_buffer(&mut rgb_buffer, (1000, 1000));
        let mut bgrx_back =
            BitMapBackend::<BGRXPixel>::with_buffer_and_format(&mut bgrx_buffer, (1000, 1000))
                .unwrap();

        rgb_back
            .draw_rect((0, 0), (1000, 1000), &BLACK, true)
            .unwrap();
        bgrx_back
            .draw_rect((0, 0), (1000, 1000), &BLACK, true)
            .unwrap();

        rgb_back
            .draw_rect(
                (0, 0),
                (1000, 1000),
                &RGBColor(0xaa, 0xbb, 0xcc).mix(0.85),
                true,
            )
            .unwrap();
        bgrx_back
            .draw_rect(
                (0, 0),
                (1000, 1000),
                &RGBColor(0xaa, 0xbb, 0xcc).mix(0.85),
                true,
            )
            .unwrap();

        rgb_back
            .draw_rect((0, 0), (1000, 1000), &RED.mix(0.85), true)
            .unwrap();
        bgrx_back
            .draw_rect((0, 0), (1000, 1000), &RED.mix(0.85), true)
            .unwrap();

        rgb_back.draw_circle((300, 300), 100, &GREEN, true).unwrap();
        bgrx_back
            .draw_circle((300, 300), 100, &GREEN, true)
            .unwrap();

        rgb_back.draw_rect((10, 10), (50, 50), &BLUE, true).unwrap();
        bgrx_back
            .draw_rect((10, 10), (50, 50), &BLUE, true)
            .unwrap();

        rgb_back
            .draw_rect((10, 10), (50, 50), &WHITE, true)
            .unwrap();
        bgrx_back
            .draw_rect((10, 10), (50, 50), &WHITE, true)
            .unwrap();

        rgb_back
            .draw_rect((10, 10), (15, 50), &YELLOW, true)
            .unwrap();
        bgrx_back
            .draw_rect((10, 10), (15, 50), &YELLOW, true)
            .unwrap();
    }

    for x in 0..1000 {
        for y in 0..1000 {
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 0] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 2] as i32)
                    .abs()
                    <= 1
            );
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 1] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 1] as i32)
                    .abs()
                    <= 1
            );
            assert!(
                (rgb_buffer[y * 3000 + x * 3 + 2] as i32
                    - bgrx_buffer[y * 4000 + x * 4 + 0] as i32)
                    .abs()
                    <= 1
            );
        }
    }
}
#[cfg(test)]
#[test]
fn test_draw_simple_lines() {
    use crate::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));
        back.draw_line((500, 0), (500, 1000), &WHITE.filled().stroke_width(5))
            .unwrap();
    }

    let nz_count = buffer.into_iter().filter(|x| *x != 0).count();

    assert_eq!(nz_count, 6 * 1000 * 3);
}

#[cfg(test)]
#[test]
fn test_bitmap_blit() {
    let src_bitmap: Vec<u8> = (0..100)
        .map(|y| (0..300).map(move |x| ((x * y) % 253) as u8))
        .flatten()
        .collect();

    use crate::prelude::*;
    let mut buffer = vec![0; 1000 * 1000 * 3];

    {
        let mut back = BitMapBackend::with_buffer(&mut buffer, (1000, 1000));
        back.blit_bitmap((500, 500), (100, 100), &src_bitmap[..])
            .unwrap();
    }

    for y in 0..1000 {
        for x in 0..1000 {
            if x >= 500 && x < 600 && y >= 500 && y < 600 {
                let lx = x - 500;
                let ly = y - 500;
                assert_eq!(buffer[y * 3000 + x * 3 + 0] as usize, (ly * lx * 3) % 253);
                assert_eq!(
                    buffer[y * 3000 + x * 3 + 1] as usize,
                    (ly * (lx * 3 + 1)) % 253
                );
                assert_eq!(
                    buffer[y * 3000 + x * 3 + 2] as usize,
                    (ly * (lx * 3 + 2)) % 253
                );
            } else {
                assert_eq!(buffer[y * 3000 + x * 3 + 0], 0);
                assert_eq!(buffer[y * 3000 + x * 3 + 1], 0);
                assert_eq!(buffer[y * 3000 + x * 3 + 2], 0);
            }
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::style::text_anchor::{HPos, Pos, VPos};
    use image::{ImageBuffer, Rgb};
    use std::fs;
    use std::path::Path;

    static DST_DIR: &str = "target/test/bitmap";

    fn checked_save_file(name: &str, content: &[u8], w: u32, h: u32) {
        /*
          Please use the PNG file to manually verify the results.
        */
        assert!(content.iter().any(|x| *x != 0));
        fs::create_dir_all(DST_DIR).unwrap();
        let file_name = format!("{}.png", name);
        let file_path = Path::new(DST_DIR).join(file_name);
        println!("{:?} created", file_path);
        let img = ImageBuffer::<Rgb<u8>, &[u8]>::from_raw(w, h, content).unwrap();
        img.save(&file_path).unwrap();
    }

    fn draw_mesh_with_custom_ticks(tick_size: i32, test_name: &str) {
        let (width, height) = (500, 500);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("This is a test", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_ranged(0..10, 0..10)
                .unwrap();

            chart
                .configure_mesh()
                .set_all_tick_mark_size(tick_size)
                .draw()
                .unwrap();
        }
        checked_save_file(test_name, &buffer, width, height);
    }

    #[test]
    fn test_draw_mesh_no_ticks() {
        draw_mesh_with_custom_ticks(0, "test_draw_mesh_no_ticks");
    }

    #[test]
    fn test_draw_mesh_negative_ticks() {
        draw_mesh_with_custom_ticks(-10, "test_draw_mesh_negative_ticks");
    }

    #[test]
    fn test_text_draw() {
        let (width, height) = (1500, 800);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let root = root
                .titled("Image Title", ("sans-serif", 60).into_font())
                .unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("All anchor point positions", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_ranged(0..100, 0..50)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .x_desc("X Axis")
                .y_desc("Y Axis")
                .draw()
                .unwrap();

            let ((x1, y1), (x2, y2), (x3, y3)) = ((-30, 30), (0, -30), (30, 30));

            for (dy, trans) in [
                FontTransform::None,
                FontTransform::Rotate90,
                FontTransform::Rotate180,
                FontTransform::Rotate270,
            ]
            .iter()
            .enumerate()
            {
                for (dx1, h_pos) in [HPos::Left, HPos::Right, HPos::Center].iter().enumerate() {
                    for (dx2, v_pos) in [VPos::Top, VPos::Center, VPos::Bottom].iter().enumerate() {
                        let x = 150_i32 + (dx1 as i32 * 3 + dx2 as i32) * 150;
                        let y = 120 + dy as i32 * 150;
                        let draw = |x, y, text| {
                            root.draw(&Circle::new((x, y), 3, &BLACK.mix(0.5))).unwrap();
                            let style = TextStyle::from(("sans-serif", 20).into_font())
                                .pos(Pos::new(*h_pos, *v_pos))
                                .transform(trans.clone());
                            root.draw_text(text, &style, (x, y)).unwrap();
                        };
                        draw(x + x1, y + y1, "dood");
                        draw(x + x2, y + y2, "dog");
                        draw(x + x3, y + y3, "goog");
                    }
                }
            }
        }
        checked_save_file("test_text_draw", &buffer, width, height);
    }

    #[test]
    fn test_text_clipping() {
        let (width, height) = (500_i32, 500_i32);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                .into_drawing_area();
            root.fill(&WHITE).unwrap();

            let style = TextStyle::from(("sans-serif", 20).into_font())
                .pos(Pos::new(HPos::Center, VPos::Center));
            root.draw_text("TOP LEFT", &style, (0, 0)).unwrap();
            root.draw_text("TOP CENTER", &style, (width / 2, 0))
                .unwrap();
            root.draw_text("TOP RIGHT", &style, (width, 0)).unwrap();

            root.draw_text("MIDDLE LEFT", &style, (0, height / 2))
                .unwrap();
            root.draw_text("MIDDLE RIGHT", &style, (width, height / 2))
                .unwrap();

            root.draw_text("BOTTOM LEFT", &style, (0, height)).unwrap();
            root.draw_text("BOTTOM CENTER", &style, (width / 2, height))
                .unwrap();
            root.draw_text("BOTTOM RIGHT", &style, (width, height))
                .unwrap();
        }
        checked_save_file("test_text_clipping", &buffer, width as u32, height as u32);
    }

    #[test]
    fn test_series_labels() {
        let (width, height) = (500, 500);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption("All series label positions", ("sans-serif", 20))
                .set_all_label_area_size(40)
                .build_ranged(0..50, 0..50)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 5, &RED)))
                .expect("Drawing error")
                .label("Series 1")
                .legend(|(x, y)| Circle::new((x, y), 3, RED.filled()));

            chart
                .draw_series(std::iter::once(Circle::new((5, 15), 10, &BLUE)))
                .expect("Drawing error")
                .label("Series 2")
                .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

            for pos in vec![
                SeriesLabelPosition::UpperLeft,
                SeriesLabelPosition::MiddleLeft,
                SeriesLabelPosition::LowerLeft,
                SeriesLabelPosition::UpperMiddle,
                SeriesLabelPosition::MiddleMiddle,
                SeriesLabelPosition::LowerMiddle,
                SeriesLabelPosition::UpperRight,
                SeriesLabelPosition::MiddleRight,
                SeriesLabelPosition::LowerRight,
                SeriesLabelPosition::Coordinate(70, 70),
            ]
            .into_iter()
            {
                chart
                    .configure_series_labels()
                    .border_style(&BLACK.mix(0.5))
                    .position(pos)
                    .draw()
                    .expect("Drawing error");
            }
        }
        checked_save_file("test_series_labels", &buffer, width, height);
    }

    #[test]
    fn test_draw_pixel_alphas() {
        let (width, height) = (100_i32, 100_i32);
        let mut buffer = vec![0; (width * height * 3) as usize];
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32))
                .into_drawing_area();
            root.fill(&WHITE).unwrap();
            for i in -20..20 {
                let alpha = i as f64 * 0.1;
                root.draw_pixel((50 + i, 50 + i), &BLACK.mix(alpha))
                    .unwrap();
            }
        }
        checked_save_file(
            "test_draw_pixel_alphas",
            &buffer,
            width as u32,
            height as u32,
        );
    }
}
