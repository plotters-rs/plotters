use super::{AsRangedCoord, Ranged, RangedCoordf64};
use std::marker::PhantomData;
use std::ops::Range;

pub trait LogScalable: Clone {
    fn as_f64(&self) -> f64;
    fn from_f64(f: f64) -> Self;
}

macro_rules! impl_log_scalable {
    (i, $t:ty) => {
        impl LogScalable for $t {
            fn as_f64(&self) -> f64 {
                if *self != 0 {
                    return *self as f64;
                }
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

pub struct LogCoord<V: LogScalable> {
    linear: RangedCoordf64,
    logic: Range<V>,
    marker: PhantomData<V>,
}

impl<V: LogScalable> Ranged for LogCoord<V> {
    type ValueType = V;

    fn map(&self, value: &V, limit: (i32, i32)) -> i32 {
        let value = value.as_f64();
        let value = value.max(self.logic.start.as_f64()).ln();
        self.linear.map(&value, limit)
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let tier_1 = (self.logic.end.as_f64() / self.logic.start.as_f64())
            .log10()
            .abs()
            .floor() as usize;
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

        let mut multiply = 10.0;
        let mut cnt = 1;
        while max_points < tier_1 / cnt {
            multiply *= 10.0;
            cnt += 1;
        }

        let mut ret = vec![];
        let mut val = (10f64).powf(self.logic.start.as_f64().log10().ceil());

        while val <= self.logic.end.as_f64() {
            ret.push(V::from_f64(val));
            for i in 1..=tier_2_density {
                let v = val
                    * (1.0 + multiply / f64::from(tier_2_density as u32 + 1) * f64::from(i as u32));
                if v > self.logic.end.as_f64() {
                    break;
                }
                ret.push(V::from_f64(v));
            }
            val *= multiply;
        }

        ret
    }

    fn range(&self) -> Range<V> {
        self.logic.clone()
    }
}
