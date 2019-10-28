use crate::drawing::backend::{BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind};
use crate::style::{Color, RGBAColor};

use image::{ImageBuffer, ImageError, Rgb, RgbImage};

use std::path::Path;

pub type BorrowedImage<'a> = ImageBuffer<Rgb<u8>, &'a mut [u8]>;

fn blend(prev: &mut u8, new: u8, a: f64) {
    *prev = ((f64::from(*prev)) * (1.0 - a) + a * f64::from(new)).min(255.0) as u8;
}

/// Macro implementation for drawing pixels, since a generic implementation would have been
/// much more unwieldy.
macro_rules! draw_pixel {
    ($img:expr, $point:expr, $color:expr) => {{
        if $point.0 as u32 >= $img.width()
            || $point.0 < 0
            || $point.1 as u32 >= $img.height()
            || $point.1 < 0
        {
            return Ok(());
        }

        let alpha = $color.alpha();
        let rgb = $color.rgb();

        if alpha >= 1.0 {
            $img.put_pixel($point.0 as u32, $point.1 as u32, Rgb([rgb.0, rgb.1, rgb.2]));
        } else {
            let pixel = $img.get_pixel_mut($point.0 as u32, $point.1 as u32);
            let new_color = [rgb.0, rgb.1, rgb.2];

            pixel.0.iter_mut().zip(&new_color).for_each(|(old, new)| {
                *old = (f64::from(*old) * (1.0 - alpha) + f64::from(*new) * alpha).min(255.0) as u8;
            });
        }
        Ok(())
    }};
}

#[cfg(feature = "gif")]
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
        ) -> Result<Self, ImageError> {
            let mut encoder = GifEncoder::new(
                File::create(path.as_ref()).map_err(ImageError::IoError)?,
                dim.0 as u16,
                dim.1 as u16,
                &[],
            )?;

            encoder.set(Repeat::Infinite)?;

            Ok(Self {
                encoder,
                width: dim.0,
                height: dim.1,
                delay: (delay + 5) / 10,
            })
        }

        pub(super) fn flush_frame(&mut self, img: &mut RgbImage) -> Result<(), ImageError> {
            let mut new_img = RgbImage::new(self.width, self.height);
            std::mem::swap(&mut new_img, img);

            let mut frame = GifFrame::from_rgb_speed(
                self.width as u16,
                self.height as u16,
                &new_img.into_raw(),
                10,
            );

            frame.delay = self.delay as u16;

            self.encoder.write_frame(&frame)?;

            Ok(())
        }
    }
}

enum Target<'a> {
    File(&'a Path, RgbImage),
    Buffer(BorrowedImage<'a>),
    #[cfg(feature = "gif")]
    Gif(Box<gif_support::GifFile>, RgbImage),
}

/// The backend that drawing a bitmap
pub struct BitMapBackend<'a> {
    /// The path to the image
    target: Target<'a>,

    /// Flag indicates if the bitmap has been saved
    saved: bool,
}

impl<'a> BitMapBackend<'a> {
    /// Create a new bitmap backend
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, dimension: (u32, u32)) -> Self {
        Self {
            target: Target::File(path.as_ref(), RgbImage::new(dimension.0, dimension.1)),
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
    #[cfg(feature = "gif")]
    pub fn gif<T: AsRef<Path>>(
        path: T,
        dimension: (u32, u32),
        frame_delay: u32,
    ) -> Result<Self, ImageError> {
        Ok(Self {
            target: Target::Gif(
                Box::new(gif_support::GifFile::new(path, dimension, frame_delay)?),
                RgbImage::new(dimension.0, dimension.1),
            ),
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
    pub fn with_buffer(buf: &'a mut [u8], dimension: (u32, u32)) -> Self {
        Self {
            target: Target::Buffer(
                BorrowedImage::from_raw(dimension.0, dimension.1, buf)
                    .expect("Buffer size must match dimensions (w * h * 3)."),
            ),
            saved: false,
        }
    }

    fn get_raw_pixel_buffer(&mut self) -> &mut [u8] {
        match &mut self.target {
            Target::File(_, img) => &mut (**img)[..],
            Target::Buffer(img) => &mut (**img)[..],
            #[cfg(feature = "gif")]
            Target::Gif(_, img) => &mut (**img)[..],
        }
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
                        base_addr.offset((begin * w * 3) as isize),
                        ((end - begin) * w * 3) as usize,
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

        for y in y0..=y1 {
            let start = (y * w as i32 + x0) as usize;
            let count = (x1 - x0 + 1) as usize;
            let mut iter = dst[(start * 3)..((start + count) * 3)].iter_mut();
            for _ in 0..(x1 - x0 + 1) {
                blend(iter.next().unwrap(), r, a);
                blend(iter.next().unwrap(), g, a);
                blend(iter.next().unwrap(), b, a);
            }
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
                    dst[(start * 3)..((start + count) * 3)]
                        .iter_mut()
                        .for_each(|e| *e = r);
                }
            } else {
                // If the entire memory block is going to be filled, just use single memset
                dst[(3 * y0 * w as i32) as usize..(3 * (y1 + 1) * w as i32) as usize]
                    .iter_mut()
                    .for_each(|e| *e = r);
            }
        } else {
            let count = (x1 - x0 + 1) as usize;
            if count < 8 {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let mut iter = dst[(start * 3)..((start + count) * 3)].iter_mut();
                    for _ in 0..(x1 - x0 + 1) {
                        *iter.next().unwrap() = r;
                        *iter.next().unwrap() = g;
                        *iter.next().unwrap() = b;
                    }
                }
            } else {
                for y in y0..=y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let start_ptr = &mut dst[start * 3] as *mut u8 as *mut (u8, u8, u8, u8, u8, u8);
                    let slice =
                        unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 2) };
                    for p in slice.iter_mut() {
                        unsafe {
                            let ptr = p as *mut (u8, u8, u8, u8, u8, u8) as *mut u64;
                            *ptr = std::mem::transmute([r, g, b, r, g, b, 0, 0]);
                        }
                    }

                    for idx in (slice.len() * 2)..count {
                        dst[start * 3 + idx * 3] = r;
                        dst[start * 3 + idx * 3 + 1] = g;
                        dst[start * 3 + idx * 3 + 2] = b;
                    }
                }
            }
        }
    }
}

impl<'a> DrawingBackend for BitMapBackend<'a> {
    type ErrorType = ImageError;

    fn get_size(&self) -> (u32, u32) {
        match &self.target {
            Target::Buffer(img) => (img.width(), img.height()),
            Target::File(_, img) => (img.width(), img.height()),
            #[cfg(feature = "gif")]
            Target::Gif(_, img) => (img.width(), img.height()),
        }
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        self.saved = false;
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<ImageError>> {
        match &mut self.target {
            Target::File(path, img) => {
                img.save(&path)
                    .map_err(|x| DrawingErrorKind::DrawingError(ImageError::IoError(x)))?;
                self.saved = true;
                Ok(())
            }
            Target::Buffer(_) => Ok(()),

            #[cfg(feature = "gif")]
            Target::Gif(target, img) => {
                target
                    .flush_frame(img)
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
    ) -> Result<(), DrawingErrorKind<ImageError>> {
        match &mut self.target {
            Target::Buffer(img) => draw_pixel!(img, point, color),
            Target::File(_, img) => draw_pixel!(img, point, color),
            #[cfg(feature = "gif")]
            Target::Gif(_, img) => draw_pixel!(img, point, color),
        }
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
                    let (w, h) = self.get_size();
                    let w = w as i32;
                    let h = h as i32;

                    // Make sure we are in the range
                    if from.0 < 0 || from.0 >= w {
                        return Ok(());
                    }

                    let dst = self.get_raw_pixel_buffer();
                    let (mut y0, mut y1) = (from.1, to.1);
                    if y0 > y1 {
                        std::mem::swap(&mut y0, &mut y1);
                    }
                    // And check the y axis isn't out of bound
                    y0 = y0.max(0);
                    y1 = y1.min(h);
                    // This is ok because once y0 > y1, there won't be any iteration anymore
                    for y in y0..=y1 {
                        dst[(y * w + from.0) as usize * 3] = r;
                        dst[(y * w + from.0) as usize * 3 + 1] = g;
                        dst[(y * w + from.0) as usize * 3 + 2] = b;
                    }
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
        src: &'b image::ImageBuffer<image::Rgb<u8>, &'b [u8]>,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (dw, dh) = self.get_size();
        let (sw, sh) = src.dimensions();

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

        let dst_start = 3 * (y0 as usize * dw as usize + x0 as usize);

        let mut dst = &mut self.get_raw_pixel_buffer()[dst_start..];

        let src_start = 3 * ((sh as i32 + y0 - y1) * sw as i32 + (sw as i32 + x0 - x1)) as usize;
        let mut src = &(**src)[src_start..];

        if src_gap == 0 && dst_gap == 0 {
            chunk_size *= num_chunks;
            num_chunks = 1;
        }
        for i in 0..num_chunks {
            dst[0..(chunk_size * 3)].copy_from_slice(&src[0..(chunk_size * 3)]);
            if i != num_chunks - 1 {
                dst = &mut dst[((chunk_size + dst_gap) * 3)..];
                src = &src[((chunk_size + src_gap) * 3)..];
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
        area.draw(&Path::new(vec![(0, 0), (10, 10)], RED.filled()))
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
fn test_draw_line_out_of_range() {
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
