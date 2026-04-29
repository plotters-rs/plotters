// pattern: Imperative Shell

//! The implementation of an actual font implementation
//!
//! This exists since for the image rendering task, we want to use
//! the system font. But in wasm application, we want the browser
//! to handle all the font issue.
//!
//! Thus we need different mechanism for the font implementation

#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
mod context;
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
mod engine;
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
mod harfrust_engine;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ab_glyph"
))]
mod migration;
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
mod system;

#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
pub(crate) use context::{push_font_context, FontContext};
#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
pub use engine::FontError;
#[cfg(all(
    not(all(target_arch = "wasm32", not(target_os = "wasi"))),
    feature = "ab_glyph"
))]
pub use migration::{register_font, InvalidFont};

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
mod web;
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
use web::FontDataInternal;

mod font_desc;
pub use font_desc::*;

/// Represents a box where a text label can be fit
pub type LayoutBox = ((i32, i32), (i32, i32));

#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
/// The type we used to represent a result of any font operations
pub type FontResult<T> = Result<T, FontError>;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
/// The error type for the font implementation
pub type FontError = <FontDataInternal as FontData>::ErrorType;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
/// The type we used to represent a result of any font operations
pub type FontResult<T> = Result<T, FontError>;

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
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
