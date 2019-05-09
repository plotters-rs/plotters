/// The implementation of an actual font implmentation
///
/// This exists since for the image rendering task, we want to use
/// the system font. But in wasm application, we want the browser
/// to handle all the font issue.
///
/// Thus we need different mechanism for the font implementation

#[cfg(not(target_arch = "wasm32"))]
mod ttf;

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports, dead_code)]
use ttf::FontDataInternal;
#[cfg(not(target_arch = "wasm32"))]
pub use ttf::FontError;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
use web::FontDataInternal;
#[cfg(target_arch = "wasm32")]
pub use web::FontError;

mod font;
pub use font::*;
