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
        // TODO: Currently it's tricky to control the labels.
        // The problem is if we need to emit the nested keypoint in this vector?
        // Once we introduce the additional metadata on the key points, we probably have better way to handle this.
        self.primary
            .key_points(max_points)
            .into_iter()
            .map(|x| (x, None))
            .collect()
    }
}

impl<P: DiscreteRanged, S: DiscreteRanged> DiscreteRanged for NestedRange<P, S> {
    fn size(&self) -> usize {
        self.secondary.iter().map(|x| x.size()).sum::<usize>()
    }

    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        let p_idx = self.primary.index_of(&value.0)?;
        let s_idx = self.secondary[p_idx].index_of(value.1.as_ref()?)?;
        Some(
            s_idx
                + self.secondary[..p_idx]
                    .iter()
                    .map(|x| x.size())
                    .sum::<usize>(),
        )
    }

    fn from_index(&self, mut index: usize) -> Option<Self::ValueType> {
        for (p_idx, snd) in self.secondary.iter().enumerate() {
            if snd.size() > index {
                return Some((
                    self.primary.from_index(p_idx).unwrap(),
                    snd.from_index(index),
                ));
            }
            index -= snd.size();
        }
        None
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nested_coord() {
        let coord = (0..10).nested_coord(|x| 0..(x + 1));

        let range = coord.range();

        assert_eq!((0, Some(0))..(10, Some(11)), range);
        assert_eq!(coord.map(&(0, None), (0, 1100)), 50);
        assert_eq!(coord.map(&(0, Some(0)), (0, 1100)), 0);
        assert_eq!(coord.map(&(5, Some(4)), (0, 1100)), 567);

        assert_eq!(coord.size(), (2 + 12) * 11 / 2);
        assert_eq!(coord.index_of(&(5, Some(4))), Some(24));
        assert_eq!(coord.from_index(24), Some((5, Some(4))));
    }
}
