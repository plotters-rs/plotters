use crate::coord::ranged1d::types::RangedCoordf64;
use crate::coord::ranged1d::{AsRangedCoord, DefaultFormatting, KeyPointHint, Ranged};
use std::marker::PhantomData;
use std::ops::Range;

/// The trait for the type that is able to be presented in the log scale.
/// This trait is primarily used by [LogRange](struct.LogRange.html).
pub trait LogScalable: Clone {
    /// Make the conversion from the type to the floating point number
    fn as_f64(&self) -> f64;
    /// Convert a floating point number to the scale
    fn from_f64(f: f64) -> Self;
}

macro_rules! impl_log_scalable {
    (i, $t:ty) => {
        impl LogScalable for $t {
            fn as_f64(&self) -> f64 {
                if *self != 0 {
                    return *self as f64;
                }
                // If this is an integer, we should allow zero point to be shown
                // on the chart, thus we can't map the zero point to inf.
                // So we just assigning a value smaller than 1 as the alternative
                // of the zero point.
                return 0.5;
            }
            fn from_f64(f: f64) -> $t {
                f.round() as $t
            }
        }
    };
    (f, $t:ty) => {
        impl LogScalable for $t {
            fn as_f64(&self) -> f64 {
                *self as f64
            }
            fn from_f64(f: f64) -> $t {
                f as $t
            }
        }
    };
}

impl_log_scalable!(i, u8);
impl_log_scalable!(i, u16);
impl_log_scalable!(i, u32);
impl_log_scalable!(i, u64);
impl_log_scalable!(f, f32);
impl_log_scalable!(f, f64);

pub trait IntoLogRange {
    type ValueType: LogScalable;
    fn log_scale(self) -> LogRange<Self::ValueType>;
}

impl<T: LogScalable> IntoLogRange for Range<T> {
    type ValueType = T;
    fn log_scale(self) -> LogRange<T> {
        LogRange(self)
    }
}

/// The logarithmic coodinate decorator.
/// This decorator is used to make the axis rendered as logarithmically.
#[derive(Clone)]
pub struct LogRange<V: LogScalable>(pub Range<V>);

impl<V: LogScalable> From<LogRange<V>> for LogCoord<V> {
    fn from(range: LogRange<V>) -> LogCoord<V> {
        LogCoord {
            linear: (range.0.start.as_f64().ln()..range.0.end.as_f64().ln()).into(),
            logic: range.0,
            marker: PhantomData,
        }
    }
}

impl<V: LogScalable> AsRangedCoord for LogRange<V> {
    type CoordDescType = LogCoord<V>;
    type Value = V;
}

/// A log scaled coordinate axis
pub struct LogCoord<V: LogScalable> {
    linear: RangedCoordf64,
    logic: Range<V>,
    marker: PhantomData<V>,
}

impl<V: LogScalable> Ranged for LogCoord<V> {
    type FormatOption = DefaultFormatting;
    type ValueType = V;

    fn map(&self, value: &V, limit: (i32, i32)) -> i32 {
        let value = value.as_f64();
        let value = value.max(self.logic.start.as_f64()).ln();
        self.linear.map(&value, limit)
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        let max_points = hint.max_num_points();
        let tier_1 = (self.logic.end.as_f64() / self.logic.start.as_f64())
            .log10()
            .abs()
            .floor()
            .max(1.0) as usize;

        let tier_2_density = if max_points < tier_1 {
            0
        } else {
            let density = 1 + (max_points - tier_1) / tier_1;
            let mut exp = 1;
            while exp * 10 <= density {
                exp *= 10;
            }
            exp - 1
        };

        let mut multiplier = 10.0;
        let mut cnt = 1;
        while max_points < tier_1 / cnt {
            multiplier *= 10.0;
            cnt += 1;
        }

        let mut ret = vec![];
        let mut val = (10f64).powf(self.logic.start.as_f64().log10().ceil());

        while val <= self.logic.end.as_f64() {
            ret.push(V::from_f64(val));
            for i in 1..=tier_2_density {
                let v = val
                    * (1.0
                        + multiplier / f64::from(tier_2_density as u32 + 1) * f64::from(i as u32));
                if v > self.logic.end.as_f64() {
                    break;
                }
                ret.push(V::from_f64(v));
            }
            val *= multiplier;
        }

        ret
    }

    fn range(&self) -> Range<V> {
        self.logic.clone()
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn regression_test_issue_143() {
        let range: LogCoord<f64> = LogRange(1.0..5.0).into();

        range.key_points(100);
    }
}
