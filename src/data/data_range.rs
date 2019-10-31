use std::cmp::{Ordering, PartialOrd};
use std::iter::IntoIterator;
use std::ops::Range;

use num_traits::{One, Zero};

/// Build a range that fits the data
pub fn fitting_range<'a, T: 'a, I: IntoIterator<Item = &'a T>>(iter: I) -> Range<T>
where
    T: Zero + One + PartialOrd + Clone,
{
    let (mut lb, mut ub) = (None, None);

    for value in iter.into_iter() {
        match lb
            .as_ref()
            .map_or(Some(Ordering::Greater), |lbv: &T| lbv.partial_cmp(value))
        {
            Some(Ordering::Greater) => lb = Some(value.clone()),
            _ => {}
        }

        match ub
            .as_ref()
            .map_or(Some(Ordering::Less), |ubv: &T| ubv.partial_cmp(value))
        {
            Some(Ordering::Less) => ub = Some(value.clone()),
            _ => {}
        }
    }

    lb.unwrap_or(Zero::zero())..ub.unwrap_or(One::one())
}
