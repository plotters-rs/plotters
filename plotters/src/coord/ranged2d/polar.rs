/*!
 The 2-dimensional polar coordinate system.
 This types of coordinate system is used by the chart constructed with [ChartBuilder::build_polar_2d](../../chart/ChartBuilder.html#method.build_polar_2d).
*/

use crate::coord::ranged1d::{KeyPointHint, Ranged, ReversibleRanged};
use crate::coord::{CoordTranslate, ReverseCoordTranslate};

use crate::style::ShapeStyle;
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use std::ops::Range;
use super::cartesian;

/// A 2D polar coordinate system described by two 1D ranged coordinate specs.
#[derive(Clone)]
pub struct Polar2d<R: Ranged, T: Ranged> {
    logic_r: R,
    logic_theta: T,
    back_r: (i32, i32),
    back_theta: (i32, i32),
}

impl<R: Ranged, T: Ranged> Polar2d<R, T> {
    /// Create a new 2D polar coordinate system
    /// - `logic_r` and `logic_t` : The description for the 1D coordinate system
    /// - `actual`: The pixel range on the screen for this coordinate system
    pub fn new<IntoR: Into<R>, IntoT: Into<T>>(
        logic_r: IntoR,
        logic_theta: IntoT,
        actual: (Range<i32>, Range<i32>),
    ) -> Self {
        Self {
            logic_r: logic_r.into(),
            logic_theta: logic_theta.into(),
            back_r: (actual.0.start, actual.0.end),
            back_theta: (actual.1.start, actual.1.end),
        }
    }
}

/// Represent a coordinate mesh for the two ranged value coordinate system
pub enum MeshLine<'a, R: Ranged, T: Ranged> {
    /// r(radius) mesh: circles (radius, 
    RMesh(BackendCoord, &'a R::ValueType),
    /// t(theta) mesh: lines (start, stop, 
    TMesh(BackendCoord, BackendCoord, &'a T::ValueType),
}

impl<'a, R: Ranged, T: Ranged> MeshLine<'a, R, T> {
    /// Draw a single mesh line onto the backend
    pub fn draw<DB: DrawingBackend>(
        &self,
        backend: &mut DB,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        match self {
            MeshLine::RMesh(radius, _) => {
                // radius mesh: circle
                // backend.draw_circle(0, radius, style)
                todo!()
            },
            MeshLine::TMesh(start, stop, _) => {
                // theta mesh: lines
                backend.draw_line(*start, *stop, style)
            },
        }
    }
}

impl<R: Ranged, T: Ranged> CoordTranslate for Polar2d<R, T> {
    type From = (R::ValueType, T::ValueType);
    fn translate(&self, from: &Self::From) -> BackendCoord {
        (
            self.logic_r.map(&from.0, self.back_r),
            self.logic_theta.map(&from.1, self.back_theta),
        )
    }
}

impl<R: ReversibleRanged, T: ReversibleRanged> ReverseCoordTranslate for Polar2d<R, T> {
    fn reverse_translate(&self, input: BackendCoord) -> Option<Self::From> {
        Some((
            self.logic_r.unmap(input.0, self.back_r)?,
            self.logic_theta.unmap(input.1, self.back_theta)?,
        ))
    }
}

//impl<R: Ranged, T: Ranged> Deref for Polar2d<R, T> {
//  type Target = Polar2d<R, T>;
//  fn deref_mut(&mut self) -> &mut Self::Target {
//      self.borrow();
//  }
//}
//
//impl<R: Ranged, T: Ranged> DerefMut for Polar2d<R, T> {
//  fn deref_mut(&mut self) -> &mut Self::Target {
//      self.borrow_mut();
//  }
//}
