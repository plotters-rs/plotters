use super::{AsRangedCoord, Ranged};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::Range;

/// The trait indicates the coordinate is discrete
/// This means we can bidirectionally map the range value to 0 to N
/// in which N is the number of distinct values of the range.
///
/// This is useful since for a histgoram, this is an abstraction of bucket.
pub trait DiscreteRanged
where
    Self: Ranged,
{
    /// Get the number of element in the range
    /// Note: we assume that all the ranged discrete coordinate has finite value
    ///
    /// - **returns** The number of values in the range
    fn size(&self) -> usize;

    /// Map a value to the index
    ///
    /// Note: This function doesn't guareentee return None when the value is out of range.
    /// The only way to confirm the value is in the range is to examing the return value isn't
    /// larger than self.size.
    ///
    /// - `value`: The value to map
    /// - **returns** The index of the value
    fn index_of(&self, value: &Self::ValueType) -> Option<usize>;

    /// Reverse map the index to the value
    ///
    /// Note: This function doesn't guareentee returning None when the index is out of range.
    ///
    /// - `value`: The index to map
    /// - **returns** The value
    fn from_index(&self, index: usize) -> Option<Self::ValueType>;

    /// Return a iterator that iterates over the all possible values
    ///
    /// - **returns** The value iterator
    fn values(&self) -> DiscreteValueIter<'_, Self>
    where
        Self: Sized,
    {
        DiscreteValueIter(self, 0, self.size())
    }

    /// Returns the previous value in this range
    ///
    /// Normally, it's based on the `from_index` and `index_of` function. But for
    /// some of the coord spec, it's possible that we value faster implementation.
    /// If this is the case, we can impelemnet the type specific impl for the `previous`
    /// and `next`.
    ///
    /// - `value`: The current value
    /// - **returns**: The value piror to current value
    fn previous(&self, value: &Self::ValueType) -> Option<Self::ValueType> {
        if let Some(idx) = self.index_of(value) {
            if idx > 0 {
                return self.from_index(idx - 1);
            }
        }
        None
    }

    /// Returns the next value in this range
    ///
    /// Normally, it's based on the `from_index` and `index_of` function. But for
    /// some of the coord spec, it's possible that we value faster implementation.
    /// If this is the case, we can impelemnet the type specific impl for the `previous`
    /// and `next`.
    ///
    /// - `value`: The current value
    /// - **returns**: The value next to current value
    fn next(&self, value: &Self::ValueType) -> Option<Self::ValueType> {
        if let Some(idx) = self.index_of(value) {
            if idx + 1 < self.size() {
                return self.from_index(idx + 1);
            }
        }
        None
    }
}

/// The axis decorator that makes key-point in the center of the value range
/// This is useful when we draw a histogram, since we want the axis value label
/// to be shown in the middle of the range rather than exactly the location where
/// the value mapped to.
pub struct CentricDiscreteRange<D: DiscreteRanged>(D);

impl<D: DiscreteRanged + Clone> Clone for CentricDiscreteRange<D> {
    fn clone(&self) -> Self {
        CentricDiscreteRange(self.0.clone())
    }
}

/// The trait for types that can decorated by `CentricDiscreteRange` decorator
pub trait IntoCentric: AsRangedCoord
where
    Self::CoordDescType: DiscreteRanged,
{
    /// Convert current ranged value into a centric ranged value
    fn into_centric(self) -> CentricDiscreteRange<Self::CoordDescType> {
        CentricDiscreteRange(self.into())
    }
}

impl<R: AsRangedCoord> IntoCentric for R where R::CoordDescType: DiscreteRanged {}

/// The value that used by the centric coordinate
#[derive(Clone)]
pub enum CentricValues<T> {
    Exact(T),
    CenterOf(T),
    Last,
}

impl<D: Debug> Debug for CentricValues<D> {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match self {
            CentricValues::Exact(value) => write!(formatter, "{:?}", value),
            CentricValues::CenterOf(value) => write!(formatter, "{:?}", value),
            CentricValues::Last => Ok(()),
        }
    }
}

impl<D: DiscreteRanged> Ranged for CentricDiscreteRange<D> {
    type ValueType = CentricValues<D::ValueType>;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let margin = ((limit.1 - limit.0) as f32 / self.0.size() as f32).round() as i32;

        match value {
            CentricValues::Exact(coord) => self.0.map(coord, (limit.0, limit.1 - margin)),
            CentricValues::CenterOf(coord) => {
                let left = self.0.map(coord, (limit.0, limit.1 - margin));
                if let Some(idx) = self.0.index_of(coord) {
                    if idx + 1 < self.0.size() {
                        let right = self.0.map(
                            &self.0.from_index(idx + 1).unwrap(),
                            (limit.0, limit.1 - margin),
                        );
                        return (left + right) / 2;
                    }
                }
                left + margin / 2
            }
            CentricValues::Last => limit.1,
        }
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        self.0
            .key_points(max_points)
            .into_iter()
            .map(CentricValues::CenterOf)
            .collect()
    }

    fn range(&self) -> Range<Self::ValueType> {
        let range = self.0.range();
        CentricValues::Exact(range.start)..CentricValues::Exact(range.end)
    }
}

impl<D: DiscreteRanged> DiscreteRanged for CentricDiscreteRange<D> {
    fn size(&self) -> usize {
        self.0.size() + 1
    }

    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        match value {
            CentricValues::Exact(value) => self.0.index_of(value),
            CentricValues::CenterOf(value) => self.0.index_of(value),
            CentricValues::Last => Some(self.0.size()),
        }
    }

    fn from_index(&self, idx: usize) -> Option<Self::ValueType> {
        if idx < self.0.size() {
            self.0.from_index(idx).map(|x| CentricValues::Exact(x))
        } else if idx == self.0.size() {
            Some(CentricValues::Last)
        } else {
            None
        }
    }
}

impl<T> From<T> for CentricValues<T> {
    fn from(this: T) -> CentricValues<T> {
        CentricValues::Exact(this)
    }
}

pub struct DiscreteValueIter<'a, T: DiscreteRanged>(&'a T, usize, usize);

impl<'a, T: DiscreteRanged> Iterator for DiscreteValueIter<'a, T> {
    type Item = T::ValueType;
    fn next(&mut self) -> Option<T::ValueType> {
        if self.1 >= self.2 {
            return None;
        }
        let idx = self.1;
        self.1 += 1;
        self.0.from_index(idx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_value_iter() {
        let range: crate::coord::numeric::RangedCoordi32 = (-10..10).into();

        let values: Vec<_> = range.values().collect();

        assert_eq!(21, values.len());

        for (expected, value) in (-10..=10).zip(values) {
            assert_eq!(expected, value);
        }
        assert_eq!(range.next(&5), Some(6));
        assert_eq!(range.next(&10), None);
        assert_eq!(range.previous(&-10), None);
        assert_eq!(range.previous(&10), Some(9));
    }

    #[test]
    fn test_centric_coord() {
        let coord = (0..10).into_centric();

        assert_eq!(coord.size(), 12);
        for i in 0..=11 {
            match coord.from_index(i as usize) {
                Some(CentricValues::Exact(value)) => assert_eq!(i, value),
                Some(CentricValues::Last) => assert_eq!(i, 11),
                _ => panic!(),
            }
        }

        for (kps, idx) in coord.key_points(20).into_iter().zip(0..) {
            match kps {
                CentricValues::CenterOf(value) if value <= 10 => assert_eq!(value, idx),
                _ => panic!(),
            }
        }

        assert_eq!(coord.map(&CentricValues::CenterOf(0), (0, 24)), 1);
        assert_eq!(coord.map(&CentricValues::Exact(0), (0, 24)), 0);
        assert_eq!(coord.map(&CentricValues::Exact(1), (0, 24)), 2);
    }
}
