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

pub struct PointSeries<'a, Coord, I: IntoIterator<Item = Coord>, E>  {
    style: ShapeStyle<'a>,
    size: u32,
    data_iter: I::IntoIter,
    make_point: &'a dyn Fn(Coord, u32, ShapeStyle<'a>) -> E,
}

impl<'a, Coord, I: IntoIterator<Item = Coord>, E> Iterator
    for PointSeries<'a, Coord, I, E>
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        return self
            .data_iter
            .next()
            .map(|x| (self.make_point)(x, self.size, self.style.clone()));
    }
}

impl<'a, Coord, I: IntoIterator<Item = Coord>, E>
    PointSeries<'a, Coord, I, E>
where
    E:PointElement<'a, Coord>
{
    pub fn new<S:Into<ShapeStyle<'a>>>(iter: I, size: u32, style: S) -> Self {
        return Self {
            data_iter: iter.into_iter(),
            size,
            style: style.into(),
            make_point: &|a,b,c|E::make_point(a,b,c),
        };
    }
}

impl<'a, Coord, I: IntoIterator<Item = Coord>, E>
    PointSeries<'a, Coord, I, E>
{
    pub fn of_element<S:Into<ShapeStyle<'a>>, F:Fn(Coord, u32, ShapeStyle<'a>) -> E>(iter: I, size: u32, style: S, cons: &'a F) -> Self {
        return Self {
            data_iter: iter.into_iter(),
            size,
            style: style.into(),
            make_point: cons
        };
    }
}
