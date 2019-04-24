use crate::element::{Path, PointElement};
use crate::style::ShapeStyle;

use std::marker::PhantomData;

pub struct LineSeries<'a, Coord, I: IntoIterator<Item = Coord>> {
    style: ShapeStyle<'a>,
    data_iter: Option<I::IntoIter>,
}

impl<'b, Coord, I: IntoIterator<Item = Coord>> Iterator for LineSeries<'b, Coord, I> {
    type Item = Path<'b, Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            return Some(Path::new(
                data_iter.unwrap().collect::<Vec<_>>(),
                self.style.clone(),
            ));
        } else {
            return None;
        }
    }
}

impl<'a, Coord, I: IntoIterator<Item = Coord>> LineSeries<'a, Coord, I> {
    pub fn new<S:Into<ShapeStyle<'a>>>(iter: I, style: S) -> Self {
        return Self {
            style: style.into(),
            data_iter: Some(iter.into_iter()),
        };
    }
}

pub struct PointSeries<'a, Coord, I: IntoIterator<Item = Coord>, P: PointElement<'a, Coord>> {
    style: ShapeStyle<'a>,
    size: u32,
    data_iter: I::IntoIter,
    pahtomdata: PhantomData<&'a P>,
}

impl<'a, Coord, I: IntoIterator<Item = Coord>, P: PointElement<'a, Coord>> Iterator
    for PointSeries<'a, Coord, I, P>
{
    type Item = P;
    fn next(&mut self) -> Option<Self::Item> {
        return self
            .data_iter
            .next()
            .map(|x| P::make_point(x, self.size, self.style.clone()));
    }
}

impl<'a, Coord, I: IntoIterator<Item = Coord>, P: PointElement<'a, Coord>>
    PointSeries<'a, Coord, I, P>
{
    pub fn new<S:Into<ShapeStyle<'a>>>(iter: I, size: u32, style: S) -> Self {
        return Self {
            data_iter: iter.into_iter(),
            size,
            style: style.into(),
            pahtomdata: PhantomData,
        };
    }
}
