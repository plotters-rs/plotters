use super::{numeric::RangedCoordusize, AsRangedCoord, DiscreteRanged, Ranged};
use std::cmp::{Ordering, PartialOrd};
use std::ops::{Add, Range};

#[derive(Clone)]
pub struct Linspace<T: Ranged, S: Clone>
where
    T::ValueType: Add<S, Output = T::ValueType> + PartialOrd + Clone,
{
    step: S,
    inner: T,
    grid_value: Vec<T::ValueType>,
}

impl<T: Ranged, S: Clone> Linspace<T, S>
where
    T::ValueType: Add<S, Output = T::ValueType> + PartialOrd + Clone,
{
    fn compute_grid_values(&mut self) {
        let range = self.inner.range();

        match (
            range.start.partial_cmp(&range.end),
            (range.start.clone() + self.step.clone()).partial_cmp(&range.end),
        ) {
            (Some(a), Some(b)) if a != b || a == Ordering::Equal || b == Ordering::Equal => (),
            (Some(a), Some(_)) => {
                let mut current = range.start;
                while current.partial_cmp(&range.end) == Some(a) {
                    self.grid_value.push(current.clone());
                    current = current + self.step.clone();
                }
            }
            _ => (),
        }
    }
}

impl<T: Ranged, S: Clone> Ranged for Linspace<T, S>
where
    T::ValueType: Add<S, Output = T::ValueType> + PartialOrd + Clone,
{
    type ValueType = T::ValueType;

    fn range(&self) -> Range<T::ValueType> {
        self.inner.range()
    }

    fn map(&self, value: &T::ValueType, limit: (i32, i32)) -> i32 {
        self.inner.map(value, limit)
    }

    fn key_points(&self, n: usize) -> Vec<T::ValueType> {
        if self.grid_value.len() == 0 {
            return vec![];
        }
        let idx_range: RangedCoordusize = (0..(self.grid_value.len() - 1)).into();

        idx_range
            .key_points(n)
            .into_iter()
            .map(|x| self.grid_value[x].clone())
            .collect()
    }
}

impl<T: Ranged, S: Clone> DiscreteRanged for Linspace<T, S>
where
    T::ValueType: Add<S, Output = T::ValueType> + PartialOrd + Clone,
{
    fn size(&self) -> usize {
        self.grid_value.len()
    }

    fn index_of(&self, value: &T::ValueType) -> Option<usize> {
        self.grid_value
            .iter()
            .position(|x| x.partial_cmp(value) == Some(Ordering::Equal))
    }

    fn from_index(&self, idx: usize) -> Option<T::ValueType> {
        self.grid_value.get(idx).map(Clone::clone)
    }
}

pub trait IntoLinspace: AsRangedCoord {
    fn step<S: Clone>(self, val: S) -> Linspace<<Self as AsRangedCoord>::CoordDescType, S>
    where
        Self::Value: Add<S, Output = Self::Value> + PartialOrd + Clone,
    {
        let mut ret = Linspace {
            step: val,
            inner: self.into(),
            grid_value: vec![],
        };

        ret.compute_grid_values();

        ret
    }
}

impl<T: AsRangedCoord> IntoLinspace for T {}
