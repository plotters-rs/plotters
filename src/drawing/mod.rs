/*!
The drawing utils for Plotter. Which handles the both low-level and high-level
drawing.

For the low-level drawing abstraction, the module defines the `DrawingBackend` trait,
which handles low-level drawing of different shapes, such as, pixels, lines, rectangles, etc.

On the top of drawing backend, one or more drawing area can be defined and different coordinate
system can be applied to the drawing areas. And the drawing area implement the high-level drawing
interface, which draws an element.

Currently we have two backend implemented:

- `BitMapBackend`: The backend that creates bitmap, this is based on `image` crate
- `SVGBackend`: The backend that creates SVG image, based on `svg` crate.

*/
mod area;
mod backend_impl;

pub mod backend;

pub use area::{DrawingArea, DrawingAreaErrorKind};
pub use backend_impl::{BitMapBackend, SVGBackend};
