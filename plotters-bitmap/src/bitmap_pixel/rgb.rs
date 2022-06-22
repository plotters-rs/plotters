use super::pixel_format::blend;
use super::PixelFormat;
use crate::BitMapBackend;
use plotters_backend::DrawingBackend;

/// The marker type that indicates we are currently using a RGB888 pixel format
pub struct RGBPixel;

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
