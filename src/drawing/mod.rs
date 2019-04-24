mod backend_impl;
mod area;

pub mod backend;
pub mod coord;

pub use backend_impl::BitMapBackend;
pub use area::{DrawingArea, DrawingAreaErrorKind};
