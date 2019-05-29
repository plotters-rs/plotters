/// The datetime coordinates
use chrono::{Date, DateTime, Duration, TimeZone};
use std::ops::Range;

use super::Ranged;

/// The ranged coordinate for date
pub struct RangedDate<Z: TimeZone>(Date<Z>, Date<Z>);

/// The ranged coordinate for the date and time
pub struct RangedDateTime<Z: TimeZone>(DateTime<Z>, DateTime<Z>);

impl<Z: TimeZone> From<Range<Date<Z>>> for RangedDate<Z> {
    fn from(range: Range<Date<Z>>) -> Self {
        Self(range.start, range.end)
    }
}

impl<Z: TimeZone> Ranged for RangedDate<Z> {
    type ValueType = Date<Z>;

    fn range(&self) -> Range<Date<Z>> {
        self.0.clone()..self.1.clone()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let total_days = (self.1.clone() - self.0.clone()).num_days() as f64;
        let value_days = (value.clone() - self.0.clone()).num_days() as f64;

        (f64::from(limit.1 - limit.0) * value_days / total_days) as i32 + limit.0
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let mut ret = vec![];

        let total_days = (self.1.clone() - self.0.clone()).num_days();
        let total_weeks = (self.1.clone() - self.0.clone()).num_weeks();

        if total_days > 0 && total_days as usize <= max_points {
            for day_idx in 0..=total_days {
                ret.push(self.0.clone() + Duration::days(day_idx));
            }
            return ret;
        }

        if total_weeks > 0 && total_weeks as usize <= max_points {
            for day_idx in 0..=total_weeks {
                ret.push(self.0.clone() + Duration::weeks(day_idx));
            }
            return ret;
        }

        ret
    }
}

impl<Z: TimeZone> super::AsRangedCoord for Range<Date<Z>> {
    type CoordDescType = RangedDate<Z>;
    type Value = Date<Z>;
}
