/// The datetime coordinates
use chrono::{Date, DateTime, Datelike, Duration, TimeZone, Timelike};
use std::ops::Range;

use super::{AsRangedCoord, DescreteRanged, Ranged};

/// The trait that describe some time value
pub trait TimeValue: Eq {
    type Tz: TimeZone;
    /// Returns the date that is no later than the time
    fn date_floor(&self) -> Date<Self::Tz>;
    /// Returns the date that is no earlier than the time
    fn date_ceil(&self) -> Date<Self::Tz>;
    /// Returns the maximum value that is eariler than the given date
    fn earliest_after_date(date: Date<Self::Tz>) -> Self;
    /// Returns the duration between two time vlaue
    fn subtract(&self, other: &Self) -> Duration;
    /// Get the timezone information for current value
    fn timezone(&self) -> Self::Tz;

    /// Map the coord
    fn map_coord(value: &Self, begin: &Self, end: &Self, limit: (i32, i32)) -> i32 {
        let total_span = end.subtract(begin);
        let value_span = value.subtract(begin);

        // First, lets try the nanoseconds precision
        if let Some(total_ns) = total_span.num_nanoseconds() {
            if let Some(value_ns) = value_span.num_nanoseconds() {
                return (f64::from(limit.1 - limit.0) * value_ns as f64 / total_ns as f64) as i32
                    + limit.0;
            }
        }

        // If it overflows, it means we have a timespan nearly 300 years, we are safe to ignore the
        // porition less than 1 day.
        let total_days = total_span.num_days() as f64;
        let value_days = value_span.num_days() as f64;

        (f64::from(limit.1 - limit.0) * value_days / total_days) as i32 + limit.0
    }
}

impl<Z: TimeZone> TimeValue for Date<Z> {
    type Tz = Z;
    fn date_floor(&self) -> Date<Z> {
        self.clone()
    }
    fn date_ceil(&self) -> Date<Z> {
        self.clone()
    }
    fn earliest_after_date(date: Date<Z>) -> Self {
        date
    }
    fn subtract(&self, other: &Date<Z>) -> Duration {
        self.clone() - other.clone()
    }
    fn timezone(&self) -> Self::Tz {
        self.timezone()
    }
}

impl<Z: TimeZone> TimeValue for DateTime<Z> {
    type Tz = Z;
    fn date_floor(&self) -> Date<Z> {
        self.date()
    }
    fn date_ceil(&self) -> Date<Z> {
        if self.time().num_seconds_from_midnight() > 0 {
            self.date() + Duration::days(1)
        } else {
            self.date()
        }
    }
    fn earliest_after_date(date: Date<Z>) -> DateTime<Z> {
        date.and_hms(0, 0, 0)
    }

    fn subtract(&self, other: &DateTime<Z>) -> Duration {
        self.clone() - other.clone()
    }
    fn timezone(&self) -> Self::Tz {
        self.timezone()
    }
}

/// The ranged coordinate for date
pub struct RangedDate<Z: TimeZone>(Date<Z>, Date<Z>);

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
        TimeValue::map_coord(value, &self.0, &self.1, limit)
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

impl<Z: TimeZone> DescreteRanged for RangedDate<Z> {
    fn next_value(this: &Date<Z>) -> Date<Z> {
        this.clone() + Duration::days(1)
    }

    fn previous_value(this: &Date<Z>) -> Date<Z> {
        this.clone() - Duration::days(1)
    }
}

impl<Z: TimeZone> AsRangedCoord for Range<Date<Z>> {
    type CoordDescType = RangedDate<Z>;
    type Value = Date<Z>;
}

/// Indicatets the coord has a monthly resolution
pub struct Monthly<T: TimeValue>(Range<T>);

impl<T: TimeValue + Clone> AsRangedCoord for Monthly<T> {
    type CoordDescType = Monthly<T>;
    type Value = T;
}

impl<T: TimeValue + Clone> Ranged for Monthly<T> {
    type ValueType = T;

    fn range(&self) -> Range<T> {
        self.0.start.clone()..self.0.end.clone()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        T::map_coord(value, &self.0.start, &self.0.end, limit)
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let start_date = self.0.start.date_ceil();
        let end_date = self.0.end.date_floor();

        let mut start_year = start_date.year();
        let mut start_month = start_date.month();
        let start_day = start_date.day();

        let end_year = end_date.year();
        let end_month = end_date.month();

        if start_day != 1 {
            start_month += 1;
            if start_month == 13 {
                start_month = 1;
                start_year += 1;
            }
        }

        let total_month = (end_year - start_year) * 12 + end_month as i32 - start_month as i32;

        fn generate_key_points<T: TimeValue>(
            mut start_year: i32,
            mut start_month: i32,
            end_year: i32,
            end_month: i32,
            step: u32,
            tz: T::Tz,
        ) -> Vec<T> {
            let mut ret = vec![];
            while end_year > start_year || (end_year == start_year && end_month >= start_month) {
                ret.push(T::earliest_after_date(tz.ymd(
                    start_year,
                    start_month as u32,
                    1,
                )));
                start_month += step as i32;

                if start_month >= 13 {
                    start_year += start_month / 12;
                    start_month %= 12;
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

impl<T: TimeValue + Clone> DescreteRanged for Monthly<T> {
    fn next_value(this: &T) -> T {
        let mut year = this.date_ceil().year();
        let mut month = this.date_ceil().month();
        month += 1;
        if month == 13 {
            month = 1;
            year += 1;
        }
        T::earliest_after_date(this.timezone().ymd(year, month, this.date_ceil().day()))
    }

    fn previous_value(this: &T) -> T {
        let mut year = this.clone().date_floor().year();
        let mut month = this.clone().date_floor().month();
        month -= 1;
        if month == 0 {
            month = 12;
            year -= 1;
        }
        T::earliest_after_date(this.timezone().ymd(year, month, this.date_floor().day()))
    }
}

/// Indicate the coord has a yearly resolution
pub struct Yearly<T: TimeValue>(Range<T>);

impl<T: TimeValue + Clone> AsRangedCoord for Yearly<T> {
    type CoordDescType = Yearly<T>;
    type Value = T;
}

fn generate_yearly_keypoints<T: TimeValue>(
    max_points: usize,
    mut start_year: i32,
    start_month: u32,
    mut end_year: i32,
    end_month: u32,
    tz: T::Tz,
) -> Vec<T> {
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
        ret.push(T::earliest_after_date(tz.ymd(start_year, start_month, 1)));
        start_year += freq as i32;
    }

    ret
}

impl<T: TimeValue + Clone> Ranged for Yearly<T> {
    type ValueType = T;

    fn range(&self) -> Range<T> {
        self.0.start.clone()..self.0.end.clone()
    }

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        T::map_coord(value, &self.0.start, &self.0.end, limit)
    }

    fn key_points(&self, max_points: usize) -> Vec<Self::ValueType> {
        let start_date = self.0.start.date_ceil();
        let end_date = self.0.end.date_floor();

        let mut start_year = start_date.year();
        let mut start_month = start_date.month();
        let start_day = start_date.day();

        let end_year = end_date.year();
        let end_month = end_date.month();

        if start_day != 1 {
            start_month += 1;
            if start_month == 13 {
                start_month = 1;
                start_year += 1;
            }
        }

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

impl<T: TimeValue + Clone> DescreteRanged for Yearly<T> {
    fn next_value(this: &T) -> T {
        T::earliest_after_date(this.timezone().ymd(this.date_floor().year() + 1, 1, 1))
    }

    fn previous_value(this: &T) -> T {
        T::earliest_after_date(this.timezone().ymd(this.date_ceil().year() - 1, 1, 1))
    }
}

/// The trait that converts a normal date coord into a yearly one
pub trait IntoMonthly<T: TimeValue> {
    fn monthly(self) -> Monthly<T>;
}

/// The trait that converts a normal date coord into a yearly one
pub trait IntoYearly<T: TimeValue> {
    fn yearly(self) -> Yearly<T>;
}

impl<T: TimeValue> IntoMonthly<T> for Range<T> {
    fn monthly(self) -> Monthly<T> {
        Monthly(self)
    }
}

impl<T: TimeValue> IntoYearly<T> for Range<T> {
    fn yearly(self) -> Yearly<T> {
        Yearly(self)
    }
}

/// The ranged coordinate for the date and time
pub struct RangedDateTime<Z: TimeZone>(DateTime<Z>, DateTime<Z>);
