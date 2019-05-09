mod svg;
pub use self::svg::SVGBackend;

#[cfg(not(target_arch = "wasm32"))]
mod bitmap;
#[cfg(not(target_arch = "wasm32"))]
pub use bitmap::BitMapBackend;
