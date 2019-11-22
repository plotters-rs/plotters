/// The implementation of an actual font implementation
///
/// This exists since for the image rendering task, we want to use
/// the system font. But in wasm application, we want the browser
/// to handle all the font issue.
///
/// Thus we need different mechanism for the font implementation

#[cfg(all(not(target_arch = "wasm32"), feature = "ttf"))]
mod ttf;
#[cfg(all(not(target_arch = "wasm32"), feature = "ttf"))]
use ttf::FontDataInternal;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "ttf")))]
mod naive;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "ttf")))]
use naive::FontDataInternal;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
use web::FontDataInternal;

mod font_desc;
pub use font_desc::*;

pub type LayoutBox = ((i32, i32), (i32, i32));

pub trait FontData: Clone {
    type ErrorType: Sized + std::error::Error + Clone;
    fn new(family: FontFamily, style: FontStyle) -> Result<Self, Self::ErrorType>;
    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType>;
    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        _pos: (i32, i32),
        _size: f64,
        _text: &str,
        _draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        panic!("The font implementation is unable to draw text");
    }
}
