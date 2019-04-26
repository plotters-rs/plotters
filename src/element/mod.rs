/// Defines the drawing elements, which is the high-level drawing interface
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use std::borrow::Borrow;

mod basic_shapes;
pub use basic_shapes::*;

mod points;
pub use points::*;

mod composable;
pub use composable::{ComposedElement, EmptyElement};

mod candlestick;
pub use candlestick::CandleStick;

/// The trait indicates it's a collection of points
pub trait PointCollection<'a, Coord> {
    /// The item in point iterator
    type Borrow: Borrow<Coord>;

    /// The point iterator
    type IntoIter: IntoIterator<Item = Self::Borrow>;

    /// framework to do the coordinate mapping
    fn point_iter(self) -> Self::IntoIter;
}

/// The trait indicates we are able to draw it on a drawing area
pub trait Drawable {
    /// Actually draws the element. The key points is already translated into the
    /// image cooridnate and can be used by DC directly
    fn draw<DB: DrawingBackend, I: Iterator<Item = BackendCoord>>(
        &self,
        pos: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>>;
}
