/*!
   The Plotters bitmap backend.

   The plotters bitmap backend allows you to render images by Plotters into bitmap.
   You can either generate image file(PNG, JPG, GIF, etc) or rendering the bitmap within internal buffer (for example for framebuffer, etc).

   See the documentation for [BitMapBackend](struct.BitMapBackend.html) for more details.
*/

#[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
mod gif_support;

mod error;
pub mod bitmap_pixel;

mod bitmap;
pub use bitmap::BitMapBackend;
pub use error::BitMapBackendError;


/*pub mod bitmap_pixel {
    pub use super::bitmap::{BGRXPixel, RGBPixel};
}*/
