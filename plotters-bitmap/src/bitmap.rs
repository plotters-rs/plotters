use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind,
};
use std::marker::PhantomData;

use crate::bitmap_pixel::{PixelFormat, RGBPixel};
use crate::error::BitMapBackendError;

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
mod image_encoding_support {
    pub(super) use image::{ImageBuffer, Rgb};
    pub(super) use std::path::Path;
    pub(super) type BorrowedImage<'a> = ImageBuffer<Rgb<u8>, &'a mut [u8]>;
}

#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
use image_encoding_support::*;

mod target;

use target::{Buffer, Target};

/// The backend that drawing a bitmap
///
/// # Warning
///
/// You should call [`.present()?`](plotters_backend::DrawingBackend::present) on a
/// `BitMapBackend`, not just `drop` it or allow it to go out of scope.
///
/// If the `BitMapBackend` is just dropped, it will make a best effort attempt to write the
/// generated charts to the output file, but any errors that occcur (such as inability to
/// create the output file) will be silently ignored.
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
            target: Target::Gif(Box::new(crate::gif_support::GifFile::new(
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
    /// When this is used, the bitmap backend will write to a user provided [u8] array (or `Vec<u8>`)
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
    pub(crate) fn get_raw_pixel_buffer(&mut self) -> &mut [u8] {
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
                        DrawingErrorKind::DrawingError(BitMapBackendError::ImageError(x))
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
        if point.0 < 0
            || point.1 < 0
            || point.0 as u32 >= self.size.0
            || point.1 as u32 >= self.size.1
        {
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
                    P::fill_rect_fast(self, from, (to.0 + 1, to.1 + 1), r, g, b);
                } else {
                    P::fill_vertical_line_fast(self, from.0, (from.1, to.1), r, g, b);
                }
            } else {
                P::blend_rect_fast(self, from, (to.0 + 1, to.1 + 1), r, g, b, alpha);
            }
            return Ok(());
        }

        plotters_backend::rasterizer::draw_line(self, from, to, style)
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
        plotters_backend::rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (sw, sh): (u32, u32),
        src: &[u8],
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
            // drop should not panic, so we ignore a failed present
            let _ = self.present();
        }
    }
}

#[cfg(test)]
mod test;
