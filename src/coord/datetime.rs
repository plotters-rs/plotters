/// The datetime coordinates
use chrono::{Date, DateTime, Datelike, Duration, TimeZone};
use std::ops::Range;

use super::{AsRangedCoord, Ranged};

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

        let week_per_point = ((total_weeks as f64) / (max_points as f64)).ceil() as usize;

        for idx in 0..=(total_weeks as usize / week_per_point) {
            ret.push(self.0.clone() + Duration::weeks((idx * week_per_point) as i64));
        }

        ret
    }
}

impl<Z: TimeZone> AsRangedCoord for Range<Date<Z>> {
    type CoordDescType = RangedDate<Z>;
    type Value = Date<Z>;
}

/// Indicatets the coord has a monthly resolution
pub struct Monthly<Z: TimeZone>(Range<Date<Z>>);

impl<Z: TimeZone> AsRangedCoord for Monthly<Z> {
    type CoordDescType = Monthly<Z>;
    type Value = Date<Z>;
}

impl<Z: TimeZone> Ranged for Monthly<Z> {
    type ValueType = Date<Z>;

    fn range(&self) -> Range<Date<Z>> {
        self.0.start.clone()..self.0.end.clone()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        let total_days = (self.0.end.clone() - self.0.start.clone()).num_days() as f64;
        let value_days = (value.clone() - self.0.start.clone()).num_days() as f64;

        (f64::from(limit.1 - limit.0) * value_days / total_days) as i32 + limit.0
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let mut start_year = self.0.start.year();
        let mut start_month = self.0.start.month();
        let start_day = self.0.start.day();

        let end_year = self.0.end.year();
        let end_month = self.0.end.month();

        if start_day != 1 {
            start_month += 1;
            if start_month == 13 {
                start_month = 1;
                start_year += 1;
            }
        }

        let total_month = (end_year - start_year) * 12 + end_month as i32 - start_month as i32;

        fn generate_key_points<Z: TimeZone>(
            mut start_year: i32,
            mut start_month: i32,
            end_year: i32,
            end_month: i32,
            step: u32,
            tz: Z,
        ) -> Vec<Date<Z>> {
            let mut ret = vec![];
            while end_year > start_year || (end_year == start_year && end_month >= start_month) {
                ret.push(tz.ymd(start_year, start_month as u32, 1));
                start_month += step as i32;

                if start_month >= 13 {
                    start_year += start_month / 12;
                    start_month = start_month % 12;
                }
            }

            ret
        }

        if total_month as usize <= max_points {
            // Monthly
            return generate_key_points(
                start_year,
                start_month as i32,
                end_year,
                end_month as i32,
                1,
                self.0.start.timezone(),
            );
        } else if total_month as usize <= max_points * 3 {
            // Quaterly
            return generate_key_points(
                start_year,
                start_month as i32,
                end_year,
                end_month as i32,
                3,
                self.0.start.timezone(),
            );
        } else if total_month as usize <= max_points * 6 {
            // Biyearly
            return generate_key_points(
                start_year,
                start_month as i32,
                end_year,
                end_month as i32,
                6,
                self.0.start.timezone(),
            );
        }

        // Otherwise we could generate the yearly keypoints
        generate_yearly_keypoints(
            max_points,
            start_year,
            start_month,
            end_year,
            end_month,
            self.0.start.timezone(),
        )
    }
}

/// Indicate the coord has a yearly resolution
pub struct Yearly<Z: TimeZone>(Range<Date<Z>>);

fn generate_yearly_keypoints<Z: TimeZone>(
    max_points: usize,
    mut start_year: i32,
    start_month: u32,
    mut end_year: i32,
    end_month: u32,
    tz: Z,
) -> Vec<Date<Z>> {
    if start_month > end_month {
        end_year -= 1;
    }

    let mut exp10 = 1;

    while (end_year - start_year + 1) as usize / (exp10 * 10) > max_points {
        exp10 *= 10;
    }

    let mut freq = exp10;

    for try_freq in &[1, 2, 5, 10] {
        freq = *try_freq * exp10;
        if (end_year - start_year + 1) as usize / (exp10 * *try_freq) <= max_points {
            break;
        }
    }

    let mut ret = vec![];

    while start_year <= end_year {
        ret.push(tz.ymd(start_year, start_month, 1));
        start_year += freq as i32;
    }

    ret
}

/// The trait that converts a normal date coord into a yearly one
pub trait IntoMonthly<Z: TimeZone> {
    fn monthly(self) -> Monthly<Z>;
}

/// The trait that converts a normal date coord into a yearly one
pub trait IntoYearly<Z: TimeZone> {
    fn yearly(self) -> Yearly<Z>;
}

impl<Z: TimeZone> IntoMonthly<Z> for Range<Date<Z>> {
    fn monthly(self) -> Monthly<Z> {
        Monthly(self)
    }
}

impl<Z: TimeZone> IntoYearly<Z> for Range<Date<Z>> {
    fn yearly(self) -> Yearly<Z> {
        Yearly(self)
    }
}
