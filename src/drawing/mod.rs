mod area;
mod backend_impl;

pub mod backend;
pub mod coord;

pub use area::{DrawingArea, DrawingAreaErrorKind};
pub use backend_impl::{BitMapBackend, SVGBackend};
