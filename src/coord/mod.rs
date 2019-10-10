/*!
Coordinate system abstractions.

Coordinate systems can be attached to drawing areas. By doing so,
the drawing area can draw elements in the guest coordinate system.
`DrawingArea::apply_coord_spec` is used to attach new coordinate system
to the drawing area.

`CoordTranslate` is the trait required by `DrawingArea::apply_coord_spec`. It provides
the forward coordinate translation: from the logic coordinate to the pixel-based absolute
backend coordinate system.

When the coordinate type implements `ReverseCoordTranslate`,
the backward translation is possible, which allows mapping pixel-based coordinate into
the logic coordinate. It's not usually used for static figure rendering, but may be useful
for a interactive figure.

`RangedCoord` is the 2D cartesian coordinate system that has two `Ranged` axis.
A ranged axis can be logarithmic and by applying an logarithmic axis, the figure is logarithmic scale.
Also, the ranged axis can be decereted, and this is required by the histogram series.

*/
use crate::drawing::backend::BackendCoord;

#[cfg(feature = "chrono")]
mod datetime;
mod logarithmic;
mod numeric;
mod ranged;

#[cfg(feature = "chrono")]
pub use datetime::{IntoMonthly, IntoYearly, RangedDate, RangedDateTime, RangedDuration};
pub use numeric::{
    RangedCoordf32, RangedCoordf64, RangedCoordi32, RangedCoordi64, RangedCoordu32, RangedCoordu64,
};
pub use ranged::{
    AsRangedCoord, DiscreteRanged, IntoCentric, IntoPartialAxis, MeshLine, Ranged, RangedCoord,
    ReversableRanged,
};

#[cfg(feature = "make_partial_axis")]
pub use ranged::make_partial_axis;

pub use logarithmic::{LogCoord, LogRange, LogScalable};

/// The trait that translates some customized object to the backend coordinate
pub trait CoordTranslate {
    type From;

    /// Translate the guest coordinate to the guest coordinate
    fn translate(&self, from: &Self::From) -> BackendCoord;
}

/// The trait indicates that the coordinate system supports reverse transform
/// This is useful when we need an interactive plot, thus we need to map the event
/// from the backend coordinate to the logical coordinate
pub trait ReverseCoordTranslate: CoordTranslate {
    /// Reverse translate the coordinate from the drawing coordinate to the
    /// logic coordinate.
    /// Note: the return value is an option, because it's possible that the drawing
    /// coordinate isn't able to be represented in te guest cooredinate system
    fn reverse_translate(&self, input: BackendCoord) -> Option<Self::From>;
}

/// The coordinate translation that only impose shift
#[derive(Debug, Clone)]
pub struct Shift(pub BackendCoord);

impl CoordTranslate for Shift {
    type From = BackendCoord;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        (from.0 + (self.0).0, from.1 + (self.0).1)
    }
}

impl ReverseCoordTranslate for Shift {
    fn reverse_translate(&self, input: BackendCoord) -> Option<BackendCoord> {
        Some((input.0 - (self.0).0, input.1 - (self.0).1))
    }
}

/// We can compose an arbitrary transformation with a shift
pub struct ShiftAndTrans<T: CoordTranslate>(Shift, T);

impl<T: CoordTranslate> CoordTranslate for ShiftAndTrans<T> {
    type From = T::From;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        let temp = self.1.translate(from);
        self.0.translate(&temp)
    }
}

impl<T: ReverseCoordTranslate> ReverseCoordTranslate for ShiftAndTrans<T> {
    fn reverse_translate(&self, input: BackendCoord) -> Option<T::From> {
        Some(self.1.reverse_translate(self.0.reverse_translate(input)?)?)
    }
}
