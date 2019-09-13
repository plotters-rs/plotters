use super::{CoordTranslate, ReverseCoordTranslate};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::ShapeStyle;

use std::ops::Range;

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

    /// This function provides the on-axis part of its range
    fn axis_pixel_range(&self, limit: (i32, i32)) -> Range<i32> {
        limit.0..limit.1
    }
}

/// The trait indicates the ranged value can be map reversely, which means
/// an pixel-based cooridinate is given, it's possible to figureout the underlying
/// logic value.
pub trait ReversableRanged: Ranged {
    fn unmap(&self, input: i32, limit: (i32, i32)) -> Option<Self::ValueType>;
}

/// The coordinate described by two ranged value
pub struct RangedCoord<X: Ranged, Y: Ranged> {
    logic_x: X,
    logic_y: Y,
    back_x: (i32, i32),
    back_y: (i32, i32),
}

impl<X: Ranged, Y: Ranged> RangedCoord<X, Y> {
    /// Create a new ranged value coordinate system
    pub fn new<IntoX: Into<X>, IntoY: Into<Y>>(
        logic_x: IntoX,
        logic_y: IntoY,
        actual: (Range<i32>, Range<i32>),
    ) -> Self {
        Self {
            logic_x: logic_x.into(),
            logic_y: logic_y.into(),
            back_x: (actual.0.start, actual.0.end),
            back_y: (actual.1.start, actual.1.end),
        }
    }

    /// Draw the mesh for the coordinate system
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

        Ok(())
    }

    /// Get the range of X axis
    pub fn get_x_range(&self) -> Range<X::ValueType> {
        self.logic_x.range()
    }

    /// Get the range of Y axis
    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        self.logic_y.range()
    }

    pub fn get_x_axis_pixel_range(&self) -> Range<i32> {
        self.logic_x.axis_pixel_range(self.back_x)
    }

    pub fn get_y_axis_pixel_range(&self) -> Range<i32> {
        self.logic_y.axis_pixel_range(self.back_y)
    }
}

impl<X: Ranged, Y: Ranged> CoordTranslate for RangedCoord<X, Y> {
    type From = (X::ValueType, Y::ValueType);

    fn translate(&self, from: &Self::From) -> BackendCoord {
        (
            self.logic_x.map(&from.0, self.back_x),
            self.logic_y.map(&from.1, self.back_y),
        )
    }
}

impl<X: ReversableRanged, Y: ReversableRanged> ReverseCoordTranslate for RangedCoord<X, Y> {
    fn reverse_translate(&self, input: BackendCoord) -> Option<Self::From> {
        Some((
            self.logic_x.unmap(input.0, self.back_x)?,
            self.logic_y.unmap(input.1, self.back_y)?,
        ))
    }
}

/// Represent a coordinate mesh for the two ranged value coordinate system
pub enum MeshLine<'a, X: Ranged, Y: Ranged> {
    XMesh(BackendCoord, BackendCoord, &'a X::ValueType),
    YMesh(BackendCoord, BackendCoord, &'a Y::ValueType),
}

impl<'a, X: Ranged, Y: Ranged> MeshLine<'a, X, Y> {
    /// Draw a single mesh line onto the backend
    pub fn draw<DB: DrawingBackend>(
        &self,
        backend: &mut DB,
        style: &ShapeStyle,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let (&left, &right) = match self {
            MeshLine::XMesh(a, b, _) => (a, b),
            MeshLine::YMesh(a, b, _) => (a, b),
        };
        backend.draw_line(left, right, &style.color)
    }
}

/// The trait indicates the coordinate is descrete, so that we can draw histogram on it
pub trait DescreteRanged
where
    Self: Ranged,
    Self::ValueType: Eq,
{
    /// Get the smallest value that is larger than the `this` value
    fn next_value(this: &Self::ValueType) -> Self::ValueType;

    /// Get the largest value that is smaller than `this` value
    fn previous_value(this: &Self::ValueType) -> Self::ValueType;
}

/// The trait for the type that can be converted into a ranged coordinate axis
pub trait AsRangedCoord: Sized {
    type CoordDescType: Ranged + From<Self>;
    type Value;
}

impl<T> AsRangedCoord for T
where
    T: Ranged,
    Range<T::ValueType>: Into<T>,
{
    type CoordDescType = T;
    type Value = T::ValueType;
}

pub struct CentricDescreteRange<D: DescreteRanged>(D)
where
    <D as Ranged>::ValueType: Eq;

pub trait IntoCentric: AsRangedCoord
where
    Self::CoordDescType: DescreteRanged,
    <Self::CoordDescType as Ranged>::ValueType: Eq,
{
    fn into_centric(self) -> CentricDescreteRange<Self::CoordDescType> {
        CentricDescreteRange(self.into())
    }
}

impl<T: AsRangedCoord> IntoCentric for T
where
    T::CoordDescType: DescreteRanged,
    <Self::CoordDescType as Ranged>::ValueType: Eq,
{
}

impl<D: DescreteRanged> Ranged for CentricDescreteRange<D>
where
    <D as Ranged>::ValueType: Eq,
{
    type ValueType = <D as Ranged>::ValueType;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let prev = <D as DescreteRanged>::previous_value(&value);
        (self.0.map(&prev, limit) + self.0.map(value, limit)) / 2
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        self.0.key_points(max_points)
    }

    fn range(&self) -> Range<Self::ValueType> {
        self.0.range()
    }
}

impl<D: DescreteRanged> DescreteRanged for CentricDescreteRange<D>
where
    <D as Ranged>::ValueType: Eq,
{
    fn next_value(this: &Self::ValueType) -> Self::ValueType {
        <D as DescreteRanged>::next_value(this)
    }

    fn previous_value(this: &Self::ValueType) -> Self::ValueType {
        <D as DescreteRanged>::previous_value(this)
    }
}

impl<D: DescreteRanged> AsRangedCoord for CentricDescreteRange<D>
where
    <D as Ranged>::ValueType: Eq,
{
    type CoordDescType = Self;
    type Value = <Self as Ranged>::ValueType;
}

pub struct PartialAxis<R: Ranged>(R, Range<R::ValueType>);

pub trait IntoPartialAxis: AsRangedCoord {
    fn partial_axis(
        self,
        axis_range: Range<<Self::CoordDescType as Ranged>::ValueType>,
    ) -> PartialAxis<Self::CoordDescType> {
        PartialAxis(self.into(), axis_range)
    }
}

impl<R: AsRangedCoord> IntoPartialAxis for R {}

impl<R: Ranged> Ranged for PartialAxis<R>
where
    R::ValueType: Clone,
{
    type ValueType = R::ValueType;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        self.0.map(value, limit)
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        self.0.key_points(max_points)
    }

    fn range(&self) -> Range<Self::ValueType> {
        self.0.range()
    }

    fn axis_pixel_range(&self, limit: (i32, i32)) -> Range<i32> {
        let left = self.map(&self.1.start, limit);
        let right = self.map(&self.1.end, limit);

        left.min(right)..left.max(right)
    }
}

impl<R: DescreteRanged> DescreteRanged for PartialAxis<R>
where
    R: Ranged,
    <R as Ranged>::ValueType: Eq + Clone,
{
    fn next_value(this: &Self::ValueType) -> Self::ValueType {
        <R as DescreteRanged>::next_value(this)
    }

    fn previous_value(this: &Self::ValueType) -> Self::ValueType {
        <R as DescreteRanged>::previous_value(this)
    }
}

impl<R: Ranged> AsRangedCoord for PartialAxis<R>
where
    <R as Ranged>::ValueType: Clone,
{
    type CoordDescType = Self;
    type Value = <Self as Ranged>::ValueType;
}

#[cfg(feature = "make_partial_axis")]
pub fn make_partial_axis<T>(
    axis_range: Range<T>,
    part: Range<f64>,
) -> Option<PartialAxis<<Range<T> as AsRangedCoord>::CoordDescType>>
where
    Range<T>: AsRangedCoord,
    T: num_traits::NumCast + Clone,
{
    let left: f64 = num_traits::cast(axis_range.start.clone())?;
    let right: f64 = num_traits::cast(axis_range.end.clone())?;

    let full_range_size = (right - left) / (part.end - part.start);

    let full_left = left - full_range_size * part.start;
    let full_right = right + full_range_size * (1.0 - part.end);

    let full_range: Range<T> = num_traits::cast(full_left)?..num_traits::cast(full_right)?;

    let axis_range: <Range<T> as AsRangedCoord>::CoordDescType = axis_range.into();

    Some(PartialAxis(full_range.into(), axis_range.range()))
}
