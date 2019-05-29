use std::collections::{hash_map::IntoIter, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::AddAssign;

use crate::coord::DescreteRanged;
use crate::element::Rectangle;
use crate::style::ShapeStyle;

/// The series that aggregate data into a histogram
pub struct Histogram<'a, XR: DescreteRanged, Y: AddAssign<Y> + Default>
where
    XR::ValueType: Eq + Hash + Default,
{
    style: ShapeStyle<'a>,
    x_margin: u32,
    iter: IntoIter<XR::ValueType, Y>,
    _p: PhantomData<XR>,
}

impl<'a, XR: DescreteRanged, Y: AddAssign<Y> + Default> Histogram<'a, XR, Y>
where
    XR::ValueType: Eq + Hash + Default,
{
    pub fn new<S: Into<ShapeStyle<'a>>, I: IntoIterator<Item = (XR::ValueType, Y)>>(
        iter: I,
        x_margin: u32,
        style: S,
    ) -> Self {
        let mut buffer = HashMap::<XR::ValueType, Y>::new();
        for (x, y) in iter.into_iter() {
            *buffer.entry(x).or_insert_with(Default::default) += y;
        }
        Self {
            style: style.into(),
            x_margin,
            iter: buffer.into_iter(),
            _p: PhantomData,
        }
    }
}

impl<'a, XR: DescreteRanged, Y: AddAssign<Y> + Default> Iterator for Histogram<'a, XR, Y>
where
    XR::ValueType: Eq + Hash + Default,
{
    type Item = Rectangle<'a, (XR::ValueType, Y)>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((x, y)) = self.iter.next() {
            let nx = XR::next_value(&x);
            let mut rect = Rectangle::new([(x, y), (nx, Y::default())], self.style.clone());
            rect.set_margin(0, 0, self.x_margin, self.x_margin);
            return Some(rect);
        }
        None
    }
}
