use super::{Ranged, AsRangedCoord, DiscreteRanged};
use super::numeric::RangedCoordusize;
use std::ops::Range;

/// The ranged value spec that needs to be grouped.
/// This is useful, for example, when we have an X axis is a integer and denotes days.
/// And we are expecting the tick mark denotes weeks, in this way we can make the range
/// spec grouping by 7 elements.
pub struct GroupBy<T:DiscreteRanged>(T, usize);

/// The trait that provides method `Self::group_by` function which creates a
/// `GroupBy` decorated ranged value.
pub trait ToGroupByRange : AsRangedCoord + Sized
where
    Self::CoordDescType : DiscreteRanged
{
    /// Make a grouping ranged value, see the documentation for `GroupBy` for details.
    ///
    /// - `value`: The number of values we want to group it
    /// - **return**: The newly created grouping range specification
    fn group_by(
        self,
        value: usize,
    ) -> GroupBy<<Self as AsRangedCoord>::CoordDescType> {
        GroupBy(self.into(), value)
    }
}

impl<T: AsRangedCoord + Sized> ToGroupByRange for T
where
    T::CoordDescType: DiscreteRanged
{
}

impl<T:DiscreteRanged> AsRangedCoord for GroupBy<T>
{
    type Value = T::ValueType;
    type CoordDescType = Self;
}

impl<T:DiscreteRanged> DiscreteRanged for GroupBy<T>
{
    fn size(&self) -> usize {
        (self.0.size() + self.1 - 1) / self.1
    }
    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        self.0.index_of(value).map(|idx| idx / self.1)
    }
    fn from_index(&self, index: usize) -> Option<Self::ValueType> {
        self.0.from_index(index * self.1)
    }
}

impl<T:DiscreteRanged> Ranged for GroupBy<T>
{
    type ValueType = T::ValueType;
    fn map(&self, value: &T::ValueType, limit: (i32, i32)) -> i32 {
        self.0.map(value, limit)
    }
    fn range(&self) -> Range<T::ValueType> {
        self.0.range()
    }
    fn key_points(&self, max_points: usize) -> Vec<T::ValueType> {
        let range = 0..(self.0.size() + self.1 - 1) / self.1;
        let logic_range: RangedCoordusize = range.into();

        logic_range
            .key_points(max_points)
            .into_iter()
            .map(|x| self.0.from_index(x * self.1).unwrap())
            .collect()
    }
}