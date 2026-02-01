use super::pixel_format::blend;
use super::PixelFormat;
use crate::BitMapBackend;
use plotters_backend::DrawingBackend;

/// The marker type that indicates we are currently using a BGRX8888 pixel format
pub struct BGRXPixel;

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
        let a = a.clamp(0.0, 1.0);
        if a == 0.0 {
            return;
        }

        let (x0, y0) = (
            upper_left.0.min(bottom_right.0).max(0),
            upper_left.1.min(bottom_right.1).max(0),
        );
        let (x1, y1) = (
            upper_left.0.max(bottom_right.0).min(w as i32),
            upper_left.1.max(bottom_right.1).min(h as i32),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 >= x1 || y0 >= y1 {
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

        for y in y0..y1 {
            let start = (y * w as i32 + x0) as usize;
            let count = (x1 - x0) as usize;

            let start_ptr = &mut dst[start * Self::PIXEL_SIZE] as *mut u8 as *mut [u8; 8];
            let slice = unsafe { std::slice::from_raw_parts_mut(start_ptr, (count - 1) / 2) };
            for rp in slice.iter_mut() {
                let ptr = rp as *mut [u8; 8] as *mut u64;
                let d1 = unsafe { ptr.read_unaligned() };
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
                    ptr.write_unaligned(h | l);
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
            upper_left.0.max(bottom_right.0).min(w as i32),
            upper_left.1.max(bottom_right.1).min(h as i32),
        );

        // This may happen when the minimal value is larger than the limit.
        // Thus we just have something that is completely out-of-range
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let dst = target.get_raw_pixel_buffer();

        if r == g && g == b {
            // If r == g == b, then we can use memset
            if x0 != 0 || x1 != w as i32 {
                // If it's not the entire row is filled, we can only do
                // memset per row
                for y in y0..y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let count = (x1 - x0) as usize;
                    dst[(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut()
                        .for_each(|e| *e = r);
                }
            } else {
                // If the entire memory block is going to be filled, just use single memset
                dst[Self::PIXEL_SIZE * (y0 * w as i32) as usize
                    ..(y1 * w as i32) as usize * Self::PIXEL_SIZE]
                    .iter_mut()
                    .for_each(|e| *e = r);
            }
        } else {
            let count = (x1 - x0) as usize;
            if count < 8 {
                for y in y0..y1 {
                    let start = (y * w as i32 + x0) as usize;
                    let mut iter = dst
                        [(start * Self::PIXEL_SIZE)..((start + count) * Self::PIXEL_SIZE)]
                        .iter_mut();
                    for _ in 0..count {
                        *iter.next().unwrap() = b;
                        *iter.next().unwrap() = g;
                        *iter.next().unwrap() = r;
                        iter.next();
                    }
                }
            } else {
                for y in y0..y1 {
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
                            let d: u64 = u64::from_ne_bytes([
                                b, g, r, 0, b, g, r, 0, // QW1
                            ]);
                            ptr.write_unaligned(d);
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
