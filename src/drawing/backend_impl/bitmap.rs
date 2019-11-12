use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, RGBAColor};
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
pub enum BitMapBackendError {
    InvalidBuffer,
    IOError(std::io::Error),
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    ImageError(ImageError),
}

impl std::fmt::Display for BitMapBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BitMapBackendError {}

fn blend(prev: &mut u8, new: u8, a: u8) {
    *prev = (*prev as i32 + (new as i32 - *prev as i32) * a as i32 / 256) as u8;
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
    fn borrow_buffer(&mut self) -> &mut [u8] {
        match self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
            Buffer::Owned(buf) => &mut buf[..],
            Buffer::Borrowed(buf) => *buf,
        }
    }
}

/// The backend that drawing a bitmap
pub struct BitMapBackend<'a> {
    /// The path to the image
    #[allow(dead_code)]
    target: Target<'a>,
    /// The size of the image
    size: (u32, u32),
    /// The data buffer of the image
    buffer: Buffer<'a>,
    /// Flag indicates if the bitmap has been saved
    saved: bool,
}

impl<'a> BitMapBackend<'a> {
    const PIXEL_SIZE: usize = 3;
}

impl<'a> BitMapBackend<'a> {
    /// Create a new bitmap backend
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, (w, h): (u32, u32)) -> Self {
        Self {
            target: Target::File(path.as_ref()),
            size: (w, h),
            buffer: Buffer::Owned(vec![0; Self::PIXEL_SIZE * (w * h) as usize]),
            saved: false,
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
        })
    }

    /// Create a new bitmap backend which only lives in-memory
    ///
    /// When this is used, the bitmap backend will write to a user provided [u8] array (or Vec<u8>).
    /// Plotters uses RGB pixel format
    ///
    /// - `buf`: The buffer to operate
    /// - `dimension`: The size of the image in pixels
    pub fn with_buffer(buf: &'a mut [u8], (w, h): (u32, u32)) -> Self {
        if (w * h) as usize * Self::PIXEL_SIZE > buf.len() {
            // TODO: This doesn't deserve a panic.
            panic!(
                "Wrong image size: H = {}, W = {}, BufSize = {}",
                w,
                h,
                buf.len()
            );
        }

        Self {
            target: Target::Buffer(PhantomData),
            size: (w, h),
            buffer: Buffer::Borrowed(buf),
            saved: false,
        }
    }

    fn get_raw_pixel_buffer(&mut self) -> &mut [u8] {
        self.buffer.borrow_buffer()
    }

    /// Split a bitmap backend vertically into several sub drawing area which allows
    /// multi-threading rendering.
    pub fn split(&mut self, area_size: &[u32]) -> Vec<BitMapBackend> {
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
                Self::with_buffer(actual_buf, (w, end - begin))
            })
            .collect()
    }

    fn blend_rect_fast(
        &mut self,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
        a: f64,
    ) {
        let (w, h) = self.get_size();
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

        let dst = self.get_raw_pixel_buffer();

        let a = (255.9 * a).floor() as u64;

        // Since we should always make sure the RGB payload occupies the logic lower bits
        // thus, this type purning should work for both LE and BE CPUs
        let (p1, p2, p3): (u64, u64, u64) = unsafe {
            std::mem::transmute([
                r as u16, b as u16, g as u16, r as u16, // QW1
                b as u16, g as u16, r as u16, b as u16, // QW2
                g as u16, r as u16, b as u16, g as u16, // QW3
            ])
        };

        let (q1, q2, q3): (u64, u64, u64) = unsafe {
            std::mem::transmute([
                g as u16, r as u16, b as u16, g as u16, // QW1
                r as u16, b as u16, g as u16, r as u16, // QW2
                b as u16, g as u16, r as u16, b as u16, // QW3
            ])
        };

        const N: u64 = 0xff00ff00ff00ff00;
        const M: u64 = 0x00ff00ff00ff00ff;

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
                h1 = (h1 * (255 - a) + q1 * a) & N;
                h2 = (h2 * (255 - a) + q2 * a) & N;
                h3 = (h3 * (255 - a) + q3 * a) & N;
                l1 = ((l1 * (255 - a) + p1 * a) & N) >> 8;
                l2 = ((l2 * (255 - a) + p2 * a) & N) >> 8;
                l3 = ((l3 * (255 - a) + p3 * a) & N) >> 8;

                unsafe {
                    *ptr = (h1 | l1, h2 | l2, h3 | l3);
                }
            }

            let mut iter = dst[((start + slice.len() * 8) * Self::PIXEL_SIZE)
                ..((start + count) * Self::PIXEL_SIZE)]
                .iter_mut();
            for _ in (slice.len() * 8)..count {
                blend(iter.next().unwrap(), r, a as u8);
                blend(iter.next().unwrap(), g, a as u8);
                blend(iter.next().unwrap(), b, a as u8);
            }
        }
    }

    fn fill_vertical_line_fast(&mut self, x: i32, ys: (i32, i32), r: u8, g: u8, b: u8) {
        let (w, h) = self.get_size();
        let w = w as i32;
        let h = h as i32;

        // Make sure we are in the range
        if x < 0 || x >= w {
            return;
        }

        let dst = self.get_raw_pixel_buffer();
        let (mut y0, mut y1) = ys;
        if y0 > y1 {
            std::mem::swap(&mut y0, &mut y1);
        }
        // And check the y axis isn't out of bound
        y0 = y0.max(0);
        y1 = y1.min(h - 1);
        // This is ok because once y0 > y1, there won't be any iteration anymore
        for y in y0..=y1 {
            dst[(y * w + x) as usize * Self::PIXEL_SIZE] = r;
            dst[(y * w + x) as usize * Self::PIXEL_SIZE + 1] = g;
            dst[(y * w + x) as usize * Self::PIXEL_SIZE + 2] = b;
        }
    }

    fn fill_rect_fast(
        &mut self,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        r: u8,
        g: u8,
        b: u8,
    ) {
        let (w, h) = self.get_size();
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

        let dst = self.get_raw_pixel_buffer();

        if r == g && g == b {
            // If r == g == b, then we can use memset
            if x0 != 0 || x1 != w as i32 - 1 {
                // If it's not the entire row is filled, we can only do
                // memset per row
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let count = (x1 - x0 + 1) as usize;
                    dst[(start * 3)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut()
                        .for_each(|e| *e = r);
                }
            } else {
                // If the entire memory block is going to be filled, just use single memset
                dst[(3 * y0 * w as i32) as usize
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
                    for _ in 0..(x1 - x0 + 1) {
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
                        dst[start * 3 + idx * Self::PIXEL_SIZE] = r;
                        dst[start * 3 + idx * Self::PIXEL_SIZE + 1] = g;
                        dst[start * 3 + idx * Self::PIXEL_SIZE + 2] = b;
                    }
                }
            }
        }
    }
}

impl<'a> DrawingBackend for BitMapBackend<'a> {
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
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<BitMapBackendError>> {
        if point.0 < 0 || point.1 < 0 {
            return Ok(());
        }

        let (w, _) = self.get_size();
        let alpha = (color.alpha() * 255.9).floor() as u8;
        let rgb = color.rgb();

        let buf = self.get_raw_pixel_buffer();

        let (x, y) = (point.0 as usize, point.1 as usize);
        let w = w as usize;

        let base = (y * w + x) * Self::PIXEL_SIZE;

        if base < buf.len() {
            unsafe {
                if alpha >= 255 {
                    *buf.get_unchecked_mut(base) = rgb.0;
                    *buf.get_unchecked_mut(base + 1) = rgb.1;
                    *buf.get_unchecked_mut(base + 2) = rgb.2;
                } else {
                    blend(buf.get_unchecked_mut(base), rgb.0, alpha);
                    blend(buf.get_unchecked_mut(base + 1), rgb.1, alpha);
                    blend(buf.get_unchecked_mut(base + 2), rgb.2, alpha);
                }
            }
        }
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let alpha = style.as_color().alpha();
        let (r, g, b) = style.as_color().rgb();

        if from.0 == to.0 || from.1 == to.1 {
            if alpha >= 1.0 {
                if from.1 == to.1 {
                    self.fill_rect_fast(from, to, r, g, b);
                } else {
                    self.fill_vertical_line_fast(from.0, (from.1, to.1), r, g, b);
                }
            } else {
                self.blend_rect_fast(from, to, r, g, b, alpha);
            }
            return Ok(());
        }

        crate::drawing::rasterizer::draw_line(self, from, to, style)
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: (i32, i32),
        bottom_right: (i32, i32),
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let alpha = style.as_color().alpha();
        let (r, g, b) = style.as_color().rgb();
        if fill {
            if alpha >= 1.0 {
                self.fill_rect_fast(upper_left, bottom_right, r, g, b);
            } else {
                self.blend_rect_fast(upper_left, bottom_right, r, g, b, alpha);
            }
            return Ok(());
        }
        crate::drawing::rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
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

impl Drop for BitMapBackend<'_> {
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
                (204, 224, 244)
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
