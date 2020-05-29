// The customized coordinate decorators.
// This file contains a set of coorindate decorators that allows you determine the
// keypoint by your own code.
use std::ops::Range;

use super::{AsRangedCoord, DiscreteRanged, KeyPointHint, Ranged};

pub struct WithKeyPoints<Inner: Ranged> {
    inner: Inner,
    bold_points: Vec<Inner::ValueType>,
    light_points: Vec<Inner::ValueType>,
}

impl<I: Ranged> WithKeyPoints<I> {
    pub fn with_light_points<T: IntoIterator<Item = I::ValueType>>(mut self, iter: T) -> Self {
        self.light_points.clear();
        self.light_points.extend(iter);
        self
    }

    pub fn bold_points(&self) -> &[I::ValueType] {
        self.bold_points.as_ref()
    }

    pub fn bold_points_mut(&mut self) -> &mut [I::ValueType] {
        self.bold_points.as_mut()
    }

    pub fn light_points(&self) -> &[I::ValueType] {
        self.light_points.as_ref()
    }

    pub fn light_points_mut(&mut self) -> &mut [I::ValueType] {
        self.light_points.as_mut()
    }
}

impl<R: Ranged> Ranged for WithKeyPoints<R>
where
    R::ValueType: Clone,
{
    type ValueType = R::ValueType;
    type FormatOption = R::FormatOption;

    fn range(&self) -> Range<Self::ValueType> {
        self.inner.range()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        self.inner.map(value, limit)
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        if hint.weight().allow_light_points() {
            self.light_points.clone()
        } else {
            self.bold_points.clone()
        }
    }

    fn axis_pixel_range(&self, limit: (i32, i32)) -> Range<i32> {
        self.inner.axis_pixel_range(limit)
    }
}

impl<R: DiscreteRanged> DiscreteRanged for WithKeyPoints<R>
where
    R::ValueType: Clone,
{
    fn size(&self) -> usize {
        self.inner.size()
    }
    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        self.inner.index_of(value)
    }
    fn from_index(&self, index: usize) -> Option<Self::ValueType> {
        self.inner.from_index(index)
    }
}

pub trait BindKeyPoints
where
    Self: AsRangedCoord,
{
    fn with_key_points(self, points: Vec<Self::Value>) -> WithKeyPoints<Self::CoordDescType> {
        WithKeyPoints {
            inner: self.into(),
            bold_points: points,
            light_points: vec![],
        }
    }
}

impl<T: AsRangedCoord> BindKeyPoints for T {}

pub struct WithKeyPointMethod<R: Ranged> {
    inner: R,
    bold_func: Box<dyn Fn(usize) -> Vec<R::ValueType>>,
    light_func: Box<dyn Fn(usize) -> Vec<R::ValueType>>,
}

pub trait BindKeyPointMethod
where
    Self: AsRangedCoord,
{
    fn with_key_point_func<F: Fn(usize) -> Vec<Self::Value> + 'static>(
        self,
        func: F,
    ) -> WithKeyPointMethod<Self::CoordDescType> {
        WithKeyPointMethod {
            inner: self.into(),
            bold_func: Box::new(func),
            light_func: Box::new(|_| vec![]),
        }
    }
}

impl<T: AsRangedCoord> BindKeyPointMethod for T {}

impl<R: Ranged> WithKeyPointMethod<R> {
    pub fn with_light_point_func<F: Fn(usize) -> Vec<R::ValueType> + 'static>(
        mut self,
        func: F,
    ) -> Self {
        self.light_func = Box::new(func);
        self
    }
}

impl<R: Ranged> Ranged for WithKeyPointMethod<R> {
    type ValueType = R::ValueType;
    type FormatOption = R::FormatOption;

    fn range(&self) -> Range<Self::ValueType> {
        self.inner.range()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        self.inner.map(value, limit)
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        if hint.weight().allow_light_points() {
            (self.light_func)(hint.max_num_points())
        } else {
            (self.bold_func)(hint.max_num_points())
        }
    }

    fn axis_pixel_range(&self, limit: (i32, i32)) -> Range<i32> {
        self.inner.axis_pixel_range(limit)
    }
}

impl<R: DiscreteRanged> DiscreteRanged for WithKeyPointMethod<R> {
    fn size(&self) -> usize {
        self.inner.size()
    }
    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        self.inner.index_of(value)
    }
    fn from_index(&self, index: usize) -> Option<Self::ValueType> {
        self.inner.from_index(index)
    }
}
