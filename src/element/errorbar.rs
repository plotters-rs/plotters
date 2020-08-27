use std::marker::PhantomData;

use crate::element::{Drawable, PointCollection};
use crate::style::ShapeStyle;
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

pub trait ErrorBarOrient<K, V> {
    type XType;
    type YType;

    fn make_coord(key: K, val: V) -> (Self::XType, Self::YType);
    fn ending_coord(coord: BackendCoord, w: u32) -> (BackendCoord, BackendCoord);
}

pub struct ErrorBarOrientH<K, V>(PhantomData<(K, V)>);

pub struct ErrorBarOrientV<K, V>(PhantomData<(K, V)>);

impl<K, V> ErrorBarOrient<K, V> for ErrorBarOrientH<K, V> {
    type XType = V;
    type YType = K;

    fn make_coord(key: K, val: V) -> (V, K) {
        (val, key)
    }

    fn ending_coord(coord: BackendCoord, w: u32) -> (BackendCoord, BackendCoord) {
        (
            (coord.0, coord.1 - w as i32 / 2),
            (coord.0, coord.1 + w as i32 / 2),
        )
    }
}

impl<K, V> ErrorBarOrient<K, V> for ErrorBarOrientV<K, V> {
    type XType = K;
    type YType = V;

    fn make_coord(key: K, val: V) -> (K, V) {
        (key, val)
    }

    fn ending_coord(coord: BackendCoord, w: u32) -> (BackendCoord, BackendCoord) {
        (
            (coord.0 - w as i32 / 2, coord.1),
            (coord.0 + w as i32 / 2, coord.1),
        )
    }
}

pub struct ErrorBar<K, V, O: ErrorBarOrient<K, V>> {
    style: ShapeStyle,
    width: u32,
    key: K,
    values: [V; 3],
    _p: PhantomData<O>,
}

impl<K, V> ErrorBar<K, V, ErrorBarOrientV<K, V>> {
    pub fn new_vertical<S: Into<ShapeStyle>>(
        key: K,
        min: V,
        avg: V,
        max: V,
        style: S,
        width: u32,
    ) -> Self {
        Self {
            style: style.into(),
            width,
            key,
            values: [min, avg, max],
            _p: PhantomData,
        }
    }
}

impl<K, V> ErrorBar<K, V, ErrorBarOrientH<K, V>> {
    pub fn new_horizontal<S: Into<ShapeStyle>>(
        key: K,
        min: V,
        avg: V,
        max: V,
        style: S,
        width: u32,
    ) -> Self {
        Self {
            style: style.into(),
            width,
            key,
            values: [min, avg, max],
            _p: PhantomData,
        }
    }
}

impl<'a, K: Clone, V: Clone, O: ErrorBarOrient<K, V>> PointCollection<'a, (O::XType, O::YType)>
    for &'a ErrorBar<K, V, O>
{
    type Point = (O::XType, O::YType);
    type IntoIter = Vec<Self::Point>;
    fn point_iter(self) -> Self::IntoIter {
        self.values
            .iter()
            .map(|v| O::make_coord(self.key.clone(), v.clone()))
            .collect()
    }
}

impl<K, V, O: ErrorBarOrient<K, V>, DB: DrawingBackend> Drawable<DB> for ErrorBar<K, V, O> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let points: Vec<_> = points.take(3).collect();

        let (from, to) = O::ending_coord(points[0], self.width);
        backend.draw_line(from, to, &self.style)?;

        let (from, to) = O::ending_coord(points[2], self.width);
        backend.draw_line(from, to, &self.style)?;

        backend.draw_line(points[0], points[2], &self.style)?;

        backend.draw_circle(points[1], self.width / 2, &self.style, self.style.filled)?;

        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_preserve_stroke_width() {
    let v = ErrorBar::new_vertical(100, 20, 50, 70, WHITE.filled().stroke_width(5), 3);
    let h = ErrorBar::new_horizontal(100, 20, 50, 70, WHITE.filled().stroke_width(5), 3);

    use crate::prelude::*;
    let da = crate::create_mocked_drawing_area(300, 300, |m| {
        m.check_draw_line(|_, w, _, _| {
            assert_eq!(w, 5);
        });
    });
    da.draw(&h).expect("Drawing Failure");
    da.draw(&v).expect("Drawing Failure");
}
