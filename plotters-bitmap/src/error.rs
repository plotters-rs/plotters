#[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
use image::ImageError;

#[derive(Debug)]
/// Indicates some error occurs within the bitmap backend
pub enum BitMapBackendError {
    /// The buffer provided is invalid, for example, wrong pixel buffer size
    InvalidBuffer,
    /// Some IO error occurs while the bitmap manipulation
    IOError(std::io::Error),
    #[cfg(all(feature = "gif", not(target_arch = "wasm32"), feature = "image"))]
    GifEncodingError(gif::EncodingError),
    #[cfg(all(not(target_arch = "wasm32"), feature = "image"))]
    /// Image encoding error
    ImageError(ImageError),
}

impl std::fmt::Display for BitMapBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for BitMapBackendError {}
