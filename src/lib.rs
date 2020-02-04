mod bitmap;
pub use bitmap::BitMapBackend;

pub mod bitmap_pixel {
    pub use super::bitmap::{BGRXPixel, PixelFormat, RGBPixel};
}
