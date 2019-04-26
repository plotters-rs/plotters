/// The abstraction of the coordinate system
use crate::drawing::backend::BackendCoord;
use std::ops::Range;

mod numeric;
mod ranged;

pub use ranged::{MeshLine, RangedCoord};
pub use numeric::{RangedCoordf32, RangedCoordf64, RangedCoordu32, RangedCoordi32, RangedCoordi64, RangedCoordu64};

/// The trait that translates some customized object to the backend coordinate
pub trait CoordTranslate {
    type From;
    fn translate(&self, from: &Self::From) -> BackendCoord;
}

/// The coordinate translation that only impose shift
#[derive(Debug, Clone)]
pub struct Shift(pub BackendCoord);

impl CoordTranslate for Shift {
    type From = BackendCoord;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        return (from.0 + (self.0).0, from.1 + (self.0).1);
    }
}

/// We can compose an abitray transformation with a shift
pub struct ShiftAndTrans<T: CoordTranslate>(Shift, T);

impl<T: CoordTranslate> CoordTranslate for ShiftAndTrans<T> {
    type From = T::From;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        let temp = self.1.translate(from);
        return self.0.translate(&temp);
    }
}

/// The trait that indicates we have a ordered and ranged value
/// Which is used to describe the axis
pub trait Ranged {
    /// The type of this value
    type ValueType;

    /// This function maps the value to i32, which is the drawing coordinate
    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32;

    /// This function gives the key points that we can draw a grid based on this
    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType>;

    /// Get the range of this value
    fn range(&self) -> Range<Self::ValueType>;
}
