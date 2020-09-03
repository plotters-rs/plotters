/*!
   The Plotters bitmap backend. 
   
   The plotters bitmap backend allows you to render images by Plotters into bitmap. 
   You can either generate image file(PNG, JPG, GIF, etc) or rendering the bitmap within internal buffer (for example for framebuffer, etc). 

   See the documentation for [BitMapBackend](struct.BitMapBackend.html) for more details.
*/

mod bitmap;
pub use bitmap::BitMapBackend;

pub mod bitmap_pixel {
    pub use super::bitmap::{BGRXPixel, PixelFormat, RGBPixel};
}
