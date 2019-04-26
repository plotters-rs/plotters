/// The abstraction of the coordinate system
use crate::drawing::backend::BackendCoord;

mod datetime;
mod numeric;
mod ranged;

pub use datetime::{RangedDate, RangedDateTime};
pub use numeric::{
    RangedCoordf32, RangedCoordf64, RangedCoordi32, RangedCoordi64, RangedCoordu32, RangedCoordu64,
};
pub use ranged::{MeshLine, Ranged, RangedCoord};

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
