use super::{AsRangedCoord, DiscreteRanged, Ranged};
use std::ops::Range;

pub struct NestedRange<Primary: DiscreteRanged, Secondary: Ranged> {
    primary: Primary,
    secondary: Vec<Secondary>,
}

impl<P: DiscreteRanged, S: Ranged> Ranged for NestedRange<P, S> {
    type ValueType = (P::ValueType, Option<S::ValueType>);

    fn range(&self) -> Range<Self::ValueType> {
        let primary_range = self.primary.range();

        let secondary_left = self.secondary[0].range().start;
        let secondary_right = self.secondary[self.primary.size() - 1].range().end;

        (primary_range.start, Some(secondary_left))..(primary_range.end, Some(secondary_right))
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let idx = self.primary.index_of(&value.0).unwrap_or(0);
        let total = self.primary.size();

        let bucket_size = (limit.1 - limit.0) / total as i32;
        let mut residual = (limit.1 - limit.0) % total as i32;

        if residual < 0 {
            residual += total as i32;
        }

        let s_left = limit.0 + bucket_size * idx as i32 + residual.min(idx as i32);
        let s_right =
            limit.0 + s_left + bucket_size + if (residual as usize) < idx { 1 } else { 0 };

        if let Some(ref secondary_value) = value.1 {
            self.secondary[idx].map(secondary_value, (s_left, s_right))
        } else {
            (s_left + s_right) / 2
        }
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        self.primary
            .key_points(max_points)
            .into_iter()
            .map(|x| (x, None))
            .collect()
    }
}

pub trait BuildNestedCoord: AsRangedCoord
where
    Self::CoordDescType: DiscreteRanged,
{
    fn nested_coord<S: AsRangedCoord>(
        self,
        builder: impl Fn(<Self::CoordDescType as Ranged>::ValueType) -> S,
    ) -> NestedRange<Self::CoordDescType, S::CoordDescType> {
        let primary: Self::CoordDescType = self.into();
        assert!(primary.size() > 0);

        let secondary = primary
            .values()
            .map(|value| builder(value).into())
            .collect();

        NestedRange {
            primary,
            secondary: secondary,
        }
    }
}

impl<T: AsRangedCoord> BuildNestedCoord for T where T::CoordDescType: DiscreteRanged {}
