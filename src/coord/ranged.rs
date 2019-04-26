use crate::drawing::backend::{DrawingBackend, DrawingErrorKind, BackendCoord};
use crate::style::ShapeStyle;
use super::{Ranged, CoordTranslate};

use std::ops::Range;

/// The coordinate description
pub struct RangedCoord<X: Ranged, Y: Ranged> {
    logic_x: X,
    logic_y: Y,
    back_x: (i32, i32),
    back_y: (i32, i32),
}

/// Represent a line in the mesh
pub enum MeshLine<'a, X: Ranged, Y: Ranged> {
    XMesh(BackendCoord, BackendCoord, &'a X::ValueType),
    YMesh(BackendCoord, BackendCoord, &'a Y::ValueType),
}

impl<'a, X: Ranged, Y: Ranged> MeshLine<'a, X, Y> {
    pub fn draw<DB: DrawingBackend>(
        &self,
        backend: &mut DB,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let (&left, &right) = match self {
            MeshLine::XMesh(a, b, _) => (a, b),
            MeshLine::YMesh(a, b, _) => (a, b),
        };
        return backend.draw_line(left, right, &Box::new(style.color));
    }
}

impl<X: Ranged, Y: Ranged> RangedCoord<X, Y> {
    pub fn new<IntoX: Into<X>, IntoY: Into<Y>>(
        logic_x: IntoX,
        logic_y: IntoY,
        actual: (Range<i32>, Range<i32>),
    ) -> Self {
        return Self {
            logic_x: logic_x.into(),
            logic_y: logic_y.into(),
            back_x: (actual.0.start, actual.0.end),
            back_y: (actual.1.start, actual.1.end),
        };
    }

    pub fn draw_mesh<E, DrawMesh: FnMut(MeshLine<X, Y>) -> Result<(), E>>(
        &self,
        h_limit: usize,
        v_limit: usize,
        mut draw_mesh: DrawMesh,
    ) -> Result<(), E> {
        let (xkp, ykp) = (
            self.logic_x.key_points(v_limit),
            self.logic_y.key_points(h_limit),
        );

        for logic_x in xkp {
            let x = self.logic_x.map(&logic_x, self.back_x);
            draw_mesh(MeshLine::XMesh(
                (x, self.back_y.0),
                (x, self.back_y.1),
                &logic_x,
            ))?;
        }

        for logic_y in ykp {
            let y = self.logic_y.map(&logic_y, self.back_y);
            draw_mesh(MeshLine::YMesh(
                (self.back_x.0, y),
                (self.back_x.1, y),
                &logic_y,
            ))?;
        }

        return Ok(());
    }

    pub fn get_x_range(&self) -> Range<X::ValueType> {
        return self.logic_x.range();
    }

    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        return self.logic_y.range();
    }
}

impl<X: Ranged, Y: Ranged> CoordTranslate for RangedCoord<X, Y> {
    type From = (X::ValueType, Y::ValueType);

    fn translate(&self, from: &Self::From) -> BackendCoord {
        return (
            self.logic_x.map(&from.0, self.back_x),
            self.logic_y.map(&from.1, self.back_y),
        );
    }
}
